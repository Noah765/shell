use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Debug},
    iter, mem,
    sync::Arc,
};

use codespan_reporting::term::{self, Config, termcolor::Buffer};
use iced::{
    Background, Border, Color, Element, Font, Length, Radius, Subscription, Task, Theme, border,
    core::text::{Highlight, IntoFragment, Span, Wrapping},
    event,
    font::{self, Family, Weight},
    futures::{SinkExt, StreamExt, channel::mpsc::Sender},
    id::Id,
    keyboard::{self, Key, key::Named},
    padding,
    platform_specific::shell::commands::layer_surface,
    runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings,
    stream,
    task::Handle,
    time::milliseconds,
    widget::{
        Container, Scrollable, column, container, operation, rich_text, rule,
        scrollable::{self, Rail, Scroller, Status},
        sensor, space, span, text, text_input,
    },
    window,
};
use numbat::{
    Context, InterpreterResult, InterpreterSettings, NumbatError,
    compact_str::{CompactString, ToCompactString},
    diagnostic::ErrorDiagnostic,
    markup::{self, FormatType, FormattedString, Formatter},
    module_importer::BuiltinModuleImporter,
    pretty_print::PrettyPrint,
    resolver::CodeSource,
    value,
};
use smithay_client_toolkit::shell::wlr_layer::{KeyboardInteractivity, Layer};
use tokio::{task, time::sleep};
use zbus::{
    Connection, proxy,
    zvariant::{self, ObjectPath, OwnedObjectPath},
};

use crate::cli::Cli;

pub struct Calculator {
    context: Arc<tokio::sync::Mutex<Context>>,
    surface_id: Option<window::Id>,
    text_input_id: Id,
    text_input_content: String,
    text_input_idle_timer: Option<Handle>,
    preview: PreviewCalculation,
    results: Vec<ResultCalculation>,
}

#[derive(Debug)]
enum PreviewCalculation {
    Pending(Handle),
    Ready(String),
}

#[derive(Debug)]
enum ResultCalculation {
    Pending,
    Success(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum CalculatorMessage {
    Toggle,
    TextInputShown,
    TextInputContentChanged(String),
    TextInputIdle,
    PreviewCalculationFinished(String),
    Submit,
    ResultCalculationFinished(usize, bool, String),
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            context: Arc::new(tokio::sync::Mutex::new(Self::construct_context())),
            surface_id: None,
            text_input_id: Id::unique(),
            text_input_content: String::new(),
            text_input_idle_timer: None,
            preview: PreviewCalculation::Ready(String::new()),
            results: Vec::new(),
        }
    }

    fn construct_context() -> Context {
        let mut context = Context::new(BuiltinModuleImporter {});
        let _ = context
            .interpret("use prelude", CodeSource::Internal)
            .unwrap();
        context.load_currency_module_on_demand(true);
        context
    }

    pub fn update(&mut self, message: CalculatorMessage) -> Task<CalculatorMessage> {
        match message {
            CalculatorMessage::Toggle => match self.surface_id {
                None => self.show(),
                Some(x) => self.hide(x),
            },
            CalculatorMessage::TextInputShown => operation::focus(self.text_input_id.clone()),
            CalculatorMessage::TextInputContentChanged(x) => {
                self.text_input_content = x;
                self.restart_idle_timer()
            }
            CalculatorMessage::TextInputIdle => {
                if let Some(x) = &self.text_input_idle_timer {
                    x.abort();
                    self.text_input_idle_timer = None;
                }
                self.display_preview()
            }
            CalculatorMessage::PreviewCalculationFinished(x) => {
                self.preview = PreviewCalculation::Ready(x);
                Task::none()
            }
            CalculatorMessage::Submit => self.submit(),
            CalculatorMessage::ResultCalculationFinished(i, success, text) => {
                if success {
                    self.results[i] = ResultCalculation::Success(text);
                } else {
                    self.results[i] = ResultCalculation::Error(text);
                }
                Task::none()
            }
        }
    }

    fn show(&mut self) -> Task<CalculatorMessage> {
        let surface_id = window::Id::unique();
        self.surface_id = Some(surface_id);

        layer_surface::get_layer_surface(SctkLayerSurfaceSettings {
            id: surface_id,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            namespace: String::from("shell-calculator"),
            size: Some((Some(1200), Some(850))),
            ..Default::default()
        })
    }

    fn hide(&mut self, surface_id: window::Id) -> Task<CalculatorMessage> {
        self.surface_id = None;
        self.text_input_content.clear();
        if let Some(x) = &self.text_input_idle_timer {
            x.abort();
            self.text_input_idle_timer = None;
        }
        self.preview = PreviewCalculation::Ready(String::new());
        layer_surface::destroy_layer_surface(surface_id)
    }

    fn restart_idle_timer(&mut self) -> Task<CalculatorMessage> {
        if let Some(x) = &self.text_input_idle_timer {
            x.abort();
        }

        let (task, handle) = Task::future(sleep(milliseconds(250)))
            .map(|_| CalculatorMessage::TextInputIdle)
            .abortable();
        self.text_input_idle_timer = Some(handle);

        task
    }

    fn display_preview(&mut self) -> Task<CalculatorMessage> {
        if let Some(ResultCalculation::Pending) = self.results.last() {
            return Task::none();
        }

        if let PreviewCalculation::Pending(x) = &self.preview {
            x.abort();
        }

        if self.text_input_content.trim_start().is_empty() {
            self.preview = PreviewCalculation::Ready(String::new());
            return Task::none();
        }

        let mut context = self.context.blocking_lock().clone();
        let input = self.text_input_content.clone();
        let (task, handle) = Task::perform(
            task::spawn_blocking(move || Self::calculate(&mut context, &input)),
            |x| CalculatorMessage::PreviewCalculationFinished(x.unwrap().unwrap_or_else(|x| x)),
        )
        .abortable();
        self.preview = PreviewCalculation::Pending(handle);

        task
    }

    fn submit(&mut self) -> Task<CalculatorMessage> {
        let context = Arc::clone(&self.context);
        let input = mem::take(&mut self.text_input_content);

        if let PreviewCalculation::Pending(x) = &self.preview {
            x.abort();
        }
        self.preview = PreviewCalculation::Ready(String::new());

        if input.trim_start().is_empty() {
            return Task::none();
        }

        let i = self.results.len();
        self.results.push(ResultCalculation::Pending);

        Task::future(async move {
            let mut context = context.lock_owned().await;
            let result = task::spawn_blocking(move || Self::calculate(&mut context, &input))
                .await
                .unwrap();
            CalculatorMessage::ResultCalculationFinished(
                i,
                result.is_ok(),
                result.unwrap_or_else(|x| x),
            )
        })
    }

    fn calculate(context: &mut Context, line: &str) -> Result<String, String> {
        let printed = Arc::new(std::sync::Mutex::new(Vec::new()));
        let printed_clone = Arc::clone(&printed);
        let mut settings = InterpreterSettings {
            print_fn: Box::new(move |x| printed_clone.lock().unwrap().push(x.clone())),
        };

        match context.interpret_with_settings(&mut settings, line, CodeSource::Text) {
            Ok((statements, mut result)) => {
                if let InterpreterResult::Value(x) = &mut result {
                    Self::shorten_value(x);
                }

                let mut output = String::new();

                for x in &statements {
                    output += &ANSIFormatter.format(&x.pretty_print(), false);
                    output += "\n";
                }

                for x in printed.lock().unwrap().iter() {
                    output += &ANSIFormatter.format(x, false);
                    output += "\n";
                }

                let result =
                    result.to_markup(statements.last(), context.dimension_registry(), true, true);
                output += ANSIFormatter.format(&result, false).trim_end();

                Ok(output)
            }
            Err(x) => match *x {
                NumbatError::ResolverError(x) => Err(Self::print_error(context, x)),
                NumbatError::NameResolutionError(x) => Err(Self::print_error(context, x)),
                NumbatError::TypeCheckError(x) => Err(Self::print_error(context, x)),
                NumbatError::RuntimeError(x) => Err(Self::print_error(context, x)),
            },
        }
    }

    fn shorten_value(value: &mut value::Value) {
        match value {
            value::Value::String(x) if x.len() > 64 => {
                x.truncate(x.floor_char_boundary(61));
                x.push_str("...");
            }
            value::Value::StructInstance(_, values) => {
                values.iter_mut().for_each(Self::shorten_value)
            }
            value::Value::List(x) => {
                let list = if x.len() > 99 {
                    x.iter()
                        .take(99)
                        .cloned()
                        .map(|mut x| {
                            Self::shorten_value(&mut x);
                            x
                        })
                        .chain(iter::once(value::Value::FormatSpecifiers(Some(
                            CompactString::const_new("..."),
                        ))))
                        .collect::<VecDeque<value::Value>>()
                } else {
                    x.iter()
                        .cloned()
                        .map(|mut x| {
                            Self::shorten_value(&mut x);
                            x
                        })
                        .collect::<VecDeque<value::Value>>()
                };

                match list.into() {
                    value::Value::List(list) => *x = list,
                    _ => panic!(),
                }
            }
            _ => {}
        }
    }

    fn print_error(context: &Context, error: impl ErrorDiagnostic) -> String {
        let mut buffer = Buffer::ansi();
        let config = Config::default();

        for x in error.diagnostics() {
            term::emit(&mut buffer, &config, &context.resolver().files, &x).unwrap();
        }

        let mut result = String::from_utf8(buffer.into_inner()).unwrap();
        result.truncate(result.trim_end().len());
        result
    }

    pub fn view(
        &self,
        cli: &Cli,
        surface_id: window::Id,
    ) -> Option<Element<'_, CalculatorMessage>> {
        if self.surface_id != Some(surface_id) {
            return None;
        }

        let view = column![
            self.view_text_input(),
            space().height(8),
            self.view_preview(cli),
            space().height(8),
            self.view_results(cli),
        ];
        Some(view.into())
    }

    fn view_text_input(&self) -> Element<'_, CalculatorMessage> {
        self.view_window(
            true,
            sensor(
                text_input("Type something here...", &self.text_input_content)
                    .id(self.text_input_id.clone())
                    .on_input(CalculatorMessage::TextInputContentChanged)
                    .on_submit(CalculatorMessage::Submit)
                    .padding(0)
                    .style(|theme: &Theme, _| text_input::Style {
                        background: Background::Color(Color::TRANSPARENT),
                        border: Border::default(),
                        icon: Color::TRANSPARENT,
                        placeholder: theme.extended_palette().secondary.base.color,
                        value: theme.palette().text,
                        selection: theme.extended_palette().primary.weak.color,
                    }),
            )
            .on_show(|_| CalculatorMessage::TextInputShown),
        )
        .padding(padding::all(12))
        .into()
    }

    fn view_preview(&self, cli: &Cli) -> Element<'_, CalculatorMessage> {
        self.view_window(
            false,
            self.view_scrollable(match &self.preview {
                PreviewCalculation::Pending(_) => text("Calculating...").into(),
                PreviewCalculation::Ready(x) => self.view_ansi_text(cli, x),
            }),
        )
        .height(Length::Fill)
        .padding(padding::all(16))
        .into()
    }

    fn view_results(&self, cli: &Cli) -> Element<'_, CalculatorMessage> {
        let view_calculation = |i| match &self.results[i] {
            ResultCalculation::Pending => text("Calculating...").into(),
            ResultCalculation::Success(x) => self.view_ansi_text(cli, x),
            ResultCalculation::Error(x) if i == self.results.len() - 1 => {
                self.view_ansi_text(cli, x)
            }
            ResultCalculation::Error(x) => self.view_ansi_text(cli, x.lines().next().unwrap()),
        };

        self.view_window(
            false,
            self.view_scrollable(column(
                (1..self.results.len())
                    .rev()
                    .flat_map(|i| {
                        [
                            view_calculation(i),
                            space().height(8).into(),
                            rule::horizontal(2).into(),
                            space().height(8).into(),
                        ]
                    })
                    .chain(self.results.first().map(|_| view_calculation(0))),
            )),
        )
        .height(Length::FillPortion(3))
        .padding(padding::all(16))
        .into()
    }

    fn view_window<'a>(
        &self,
        focused: bool,
        content: impl Into<Element<'a, CalculatorMessage>>,
    ) -> Container<'a, CalculatorMessage> {
        container(content).style(move |theme| container::Style {
            background: Some(Background::Color(theme.palette().background)),
            border: Border {
                color: if focused {
                    theme.palette().primary
                } else {
                    theme.extended_palette().background.strong.color
                },
                width: 1.0,
                radius: Radius::new(12),
            },
            ..Default::default()
        })
    }

    fn view_scrollable<'a>(
        &self,
        content: impl Into<Element<'a, CalculatorMessage>>,
    ) -> Element<'a, CalculatorMessage> {
        Scrollable::new(content)
            .width(Length::Fill)
            .spacing(16)
            .scrollbar_width(4)
            .scroller_width(8)
            .style(|theme, status| {
                let palette = theme.extended_palette();
                scrollable::Style {
                    vertical_rail: Rail {
                        background: Some(palette.background.weak.color.into()),
                        border: border::rounded(8),
                        scroller: Scroller {
                            background: match status {
                                Status::Hovered {
                                    is_vertical_scrollbar_hovered: true,
                                    ..
                                } => palette.primary.strong.color.into(),
                                Status::Dragged {
                                    is_vertical_scrollbar_dragged: true,
                                    ..
                                } => palette.primary.base.color.into(),
                                _ => palette.background.strongest.color.into(),
                            },
                            border: border::rounded(8),
                        },
                    },
                    ..scrollable::default(theme, status)
                }
            })
            .into()
    }

    fn view_ansi_text<'a>(&self, cli: &Cli, text: &'a str) -> Element<'a, CalculatorMessage> {
        let mut bold = false;
        let mut dimmed = false;
        let mut italic = false;
        let mut underline = false;
        let mut strikethrough = false;
        let mut fg_color = None;
        let mut bg_color = None;

        let first_span = text
            .split_once("\x1b[")
            .map_or_else(|| span(text), |x| span(x.0));

        let spans = text.split_terminator("\x1b[").skip(1).filter_map(|x| {
            let (attribute, text) = x.split_once('m').unwrap();

            match attribute.split(";").collect::<Vec<_>>()[..] {
                ["0"] => {
                    bold = false;
                    dimmed = false;
                    italic = false;
                    underline = false;
                    strikethrough = false;
                    fg_color = None;
                    bg_color = None;
                }
                ["1"] => bold = true,
                ["2"] => dimmed = true,
                ["3"] => italic = true,
                ["4"] => underline = true,
                ["9"] => strikethrough = true,
                ["30"] | ["38", "5", "0"] => fg_color = Some(cli.background_color),
                ["31"] | ["38", "5", "1" | "9"] => fg_color = Some(cli.red),
                ["32"] | ["38", "5", "2" | "10"] => fg_color = Some(cli.green),
                ["33"] | ["38", "5", "3" | "11"] => fg_color = Some(cli.yellow),
                ["34"] | ["38", "5", "4" | "12"] => fg_color = Some(cli.blue),
                ["35"] | ["38", "5", "5" | "13"] => fg_color = Some(cli.magenta),
                ["36"] | ["38", "5", "6" | "14"] => fg_color = Some(cli.cyan),
                ["40"] | ["48", "5", "0"] => bg_color = Some(cli.background_color),
                ["41"] | ["48", "5", "1" | "9"] => bg_color = Some(cli.red),
                ["42"] | ["48", "5", "2" | "10"] => bg_color = Some(cli.green),
                ["43"] | ["48", "5", "3" | "11"] => bg_color = Some(cli.yellow),
                ["44"] | ["48", "5", "4" | "12"] => bg_color = Some(cli.blue),
                ["45"] | ["48", "5", "5" | "13"] => bg_color = Some(cli.magenta),
                ["46"] | ["48", "5", "6" | "14"] => bg_color = Some(cli.cyan),
                _ => panic!("Unsupported ANSI attribute: {attribute:?}"),
            }

            if text.is_empty() {
                return None;
            }

            Some(Span {
                text: text.into_fragment(),
                font: Some(Font {
                    family: Family::Monospace,
                    weight: if bold {
                        Weight::Bold
                    } else if dimmed {
                        Weight::Thin
                    } else {
                        Weight::Medium
                    },
                    style: if italic {
                        font::Style::Italic
                    } else {
                        font::Style::Normal
                    },
                    ..Default::default()
                }),
                color: fg_color,
                highlight: bg_color.map(|x| Highlight {
                    background: Background::Color(x),
                    border: Border::default(),
                }),
                underline,
                strikethrough,
                ..Default::default()
            })
        });
        let spans: Vec<Span> = iter::once(first_span).chain(spans).collect();

        rich_text(spans).wrapping(Wrapping::WordOrGlyph).into()
    }

    pub fn subscription(&self) -> Subscription<CalculatorMessage> {
        Subscription::batch([
            self.escape_subscription(),
            Subscription::run(|| stream::channel(64, Self::global_shortcuts)),
        ])
    }

    fn escape_subscription(&self) -> Subscription<CalculatorMessage> {
        event::listen_with(|event, _, _| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                ..
            }) => Some(CalculatorMessage::Toggle),
            _ => None,
        })
    }

    async fn global_shortcuts(mut sender: Sender<CalculatorMessage>) {
        let connection = Connection::session().await.unwrap();

        let shortcuts = GlobalShortcutsProxy::new(&connection).await.unwrap();
        let name = connection
            .unique_name()
            .unwrap()
            .strip_prefix(":")
            .unwrap()
            .replace(".", "_");

        let mut create_session_response_stream = RequestProxy::builder(&connection)
            .path(format!(
                "/org/freedesktop/portal/desktop/request/{name}/create_session"
            ))
            .unwrap()
            .build()
            .await
            .unwrap()
            .receive_response()
            .await
            .unwrap();

        let handle_token = zvariant::Value::from("create_session");
        let session_handle_token = zvariant::Value::from("shell");
        let session_options = HashMap::from([
            ("handle_token", &handle_token),
            ("session_handle_token", &session_handle_token),
        ]);
        shortcuts.create_session(session_options).await.unwrap();

        let response = create_session_response_stream.next().await.unwrap();
        let session_handle = response
            .args()
            .unwrap()
            .results
            .remove("session_handle")
            .unwrap();
        let session_handle = match &session_handle {
            zvariant::Value::Str(x) => ObjectPath::from_str_unchecked(x),
            _ => panic!(),
        };

        let description = zvariant::Value::from("Toggle the calculator");
        let data = [&(
            "toggleCalculator",
            HashMap::from([("description", &description)]),
        )];
        shortcuts
            .bind_shortcuts(&session_handle, &data, "", HashMap::new())
            .await
            .unwrap();

        let mut activation_stream = shortcuts
            .receive_activated_with_args(&[(1, "toggleCalculator")])
            .await
            .unwrap();
        while let Some(x) = activation_stream.next().await {
            if x.args().unwrap().session_handle == session_handle {
                sender.send(CalculatorMessage::Toggle).await.unwrap();
            }
        }
    }
}

impl Debug for Calculator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Calculator")
            .field("context", &"Context { .. }")
            .field("surface_id", &self.surface_id)
            .field("text_input_id", &self.text_input_id)
            .field("text_input_content", &self.text_input_content)
            .field("text_input_idle_timer", &self.text_input_idle_timer)
            .field("preview", &self.preview)
            .field("results", &self.results)
            .finish()
    }
}

#[proxy(
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop",
    interface = "org.freedesktop.portal.GlobalShortcuts"
)]
trait GlobalShortcuts {
    fn create_session(
        &self,
        options: HashMap<&str, &zvariant::Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    fn bind_shortcuts(
        &self,
        session_handle: &ObjectPath<'_>,
        shortcuts: &[&(&str, HashMap<&str, &zvariant::Value<'_>>)],
        parent_window: &str,
        options: HashMap<&str, &zvariant::Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    #[zbus(signal)]
    fn activated(
        &self,
        session_handle: ObjectPath<'_>,
        shortcut_id: &str,
        timestamp: u64,
        options: HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.portal.Desktop",
    interface = "org.freedesktop.portal.Request",
    assume_defaults = true
)]
trait Request {
    #[zbus(signal)]
    fn response(
        &self,
        response: u32,
        results: HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<()>;
}

struct ANSIFormatter;

impl markup::Formatter for ANSIFormatter {
    fn format_part(&self, part: &FormattedString) -> CompactString {
        let text = &part.2;
        let result = match part.1 {
            FormatType::Whitespace => text.to_string(),
            FormatType::Emphasized => format!("\x1b[1m{text}\x1b[0m"),
            FormatType::Dimmed => format!("\x1b[2m{text}\x1b[0m"),
            FormatType::Text => text.to_string(),
            FormatType::String => format!("\x1b[32m{text}\x1b[0m"),
            FormatType::Keyword => format!("\x1b[35m{text}\x1b[0m"),
            FormatType::Value => format!("\x1b[33m{text}\x1b[0m"),
            FormatType::Unit => format!("\x1b[36m{text}\x1b[0m"),
            FormatType::Identifier => text.to_string(),
            FormatType::TypeIdentifier => format!("\x1b[34m\x1b[3m{text}\x1b[0m"),
            FormatType::Operator => format!("\x1b[1m{text}\x1b[0m"),
            FormatType::Decorator => format!("\x1b[32m{text}\x1b[0m"),
        };
        result.to_compact_string()
    }
}
