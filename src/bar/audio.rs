use std::{cell::RefCell, collections::HashMap, rc::Rc, thread};

use iced::{
    Element, Subscription,
    alignment::Horizontal,
    widget::{column, text},
};
use pipewire::{
    context::ContextRc,
    main_loop::MainLoopRc,
    metadata::{Metadata, MetadataListener},
    node::{Node, NodeListener},
    registry::{GlobalObject, RegistryRc},
    spa::{
        param::ParamType,
        pod::{Pod, Value, ValueArray, deserialize::PodDeserializer},
        sys,
        utils::dict::DictRef,
    },
    types::ObjectType,
};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;

use crate::{bar::BAR_WIDTH, icon};

#[derive(Debug)]
pub struct Audio(Option<u8>);

#[derive(Clone, Debug)]
pub struct AudioMessage(Option<u8>);

impl Audio {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn update(&mut self, message: AudioMessage) {
        self.0 = message.0;
    }

    pub fn view(&self, width: f32) -> Element<'_, AudioMessage> {
        let size = 16.0 / BAR_WIDTH * width;
        let icon = match self.0 {
            None => icon::volume_mute().size(size).line_height(1.1),
            Some(0) => icon::volume_1().size(size).line_height(1.1),
            Some(1..34) => icon::volume_2().size(size).line_height(1.1),
            Some(34..67) => icon::volume_3().size(size).line_height(1.1),
            Some(67..) => icon::volume_4().size(size).line_height(1.1),
        };

        let text = text!("{}%", self.0.unwrap_or(0))
            .size(10.0 / BAR_WIDTH * width)
            .line_height(1.0);

        column![icon, text].align_x(Horizontal::Center).into()
    }

    pub fn subscription(&self) -> Subscription<AudioMessage> {
        Subscription::run(|| {
            let (sender, receiver) = mpsc::channel(64);
            thread::spawn(|| AudioThread::run(sender));
            ReceiverStream::new(receiver)
        })
    }
}

struct AudioThread {
    sender: Sender<AudioMessage>,
    registry: RegistryRc,
    default: Option<(Metadata, MetadataListener)>,
    sinks: Vec<Sink>,
    default_sink_name: Option<String>,
    default_sink_listener: Option<NodeListener>,
}

struct Sink {
    id: u32,
    name: String,
    node: Node,
}

impl AudioThread {
    fn run(sender: Sender<AudioMessage>) {
        let main_loop = MainLoopRc::new(None).unwrap();
        let context = ContextRc::new(&main_loop, None).unwrap();
        let core = context.connect_rc(None).unwrap();
        let registry = core.get_registry_rc().unwrap();

        let this = Rc::new(RefCell::new(Self {
            sender,
            registry: RegistryRc::clone(&registry),
            default: None,
            sinks: Vec::new(),
            default_sink_name: None,
            default_sink_listener: None,
        }));
        let this_clone = Rc::clone(&this);

        let _registry_listener = registry
            .add_listener_local()
            .global(move |x| Self::handle_add_global_object(Rc::clone(&this), x))
            .global_remove(move |x| this_clone.borrow_mut().handle_remove_global_object(x))
            .register();

        main_loop.run();
    }

    fn handle_add_global_object(this: Rc<RefCell<Self>>, object: &GlobalObject<&DictRef>) {
        if object.type_ == ObjectType::Metadata
            && object.props.unwrap().get("metadata.name") == Some("default")
        {
            Self::handle_default(this, object);
        } else if object.type_ == ObjectType::Node
            && object.props.unwrap().get("media.class") == Some("Audio/Sink")
        {
            this.borrow_mut().handle_sink(object);
        }
    }

    fn handle_default(this: Rc<RefCell<Self>>, object: &GlobalObject<&DictRef>) {
        let metadata: Metadata = this.borrow().registry.bind(object).unwrap();

        let this_clone = Rc::clone(&this);
        let listener = metadata
            .add_listener_local()
            .property(move |_, key, _, value| {
                this_clone.borrow_mut().handle_default_property(key, value)
            })
            .register();

        this.borrow_mut().default = Some((metadata, listener))
    }

    fn handle_default_property(&mut self, key: Option<&str>, value: Option<&str>) -> i32 {
        if key != Some("default.audio.sink") {
            return 0;
        }

        let name = serde_json::from_str::<HashMap<&str, _>>(value.unwrap())
            .unwrap()
            .remove("name")
            .unwrap();

        if let Some(i) = self.sinks.iter().position(|x| x.name == name) {
            self.listen_to_default_sink_at(i);
        } else {
            self.default_sink_listener = None;
        }

        self.default_sink_name = Some(name);
        0
    }

    fn handle_sink(&mut self, object: &GlobalObject<&DictRef>) {
        self.sinks.push(Sink {
            id: object.id,
            name: object.props.unwrap().get("node.name").unwrap().to_string(),
            node: self.registry.bind(object).unwrap(),
        });
        let i = self.sinks.len() - 1;

        if self
            .default_sink_name
            .as_ref()
            .is_some_and(|x| *x == self.sinks[i].name)
        {
            self.listen_to_default_sink_at(i);
        }
    }

    fn listen_to_default_sink_at(&mut self, index: usize) {
        let node = &self.sinks[index].node;

        let sender = self.sender.clone();
        let listener = node
            .add_listener_local()
            .param(move |_, _, _, _, pod| {
                Self::handle_default_sink_props(pod.unwrap(), sender.clone())
            })
            .register();
        node.subscribe_params(&[ParamType::Props]);

        self.default_sink_listener = Some(listener);
    }

    fn handle_default_sink_props(props: &Pod, sender: Sender<AudioMessage>) {
        let object = match PodDeserializer::deserialize_any_from(props.as_bytes()) {
            Ok((_, Value::Object(x))) => x,
            _ => panic!(),
        };

        let mut mute = None;
        let mut volume = None;
        for x in object.properties {
            match (x.key, x.value) {
                (sys::SPA_PROP_mute, Value::Bool(x)) => mute = Some(x),
                (sys::SPA_PROP_channelVolumes, Value::ValueArray(ValueArray::Float(x))) => {
                    volume = Some((x[0].powf(1_f32 / 3_f32) * 100_f32).round() as u8);
                }
                _ => {}
            }
        }

        let volume = match (mute, volume) {
            (Some(true), Some(_)) => None,
            (Some(false), Some(x)) => Some(x),
            _ => return,
        };

        sender.blocking_send(AudioMessage(volume)).unwrap();
    }

    fn handle_remove_global_object(&mut self, id: u32) {
        let Some(i) = self.sinks.iter().position(|x| x.id == id) else {
            return;
        };

        let sink = self.sinks.swap_remove(i);

        if Some(sink.name) == self.default_sink_name {
            self.default_sink_name = None;
            self.default_sink_listener = None;
        }
    }
}
