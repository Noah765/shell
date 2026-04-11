use std::{cell::RefCell, thread};

use iced::{
    Element, Subscription,
    alignment::Horizontal,
    widget::{column, text},
};
use pipewire::{
    context::ContextRc,
    keys,
    main_loop::MainLoopRc,
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
            thread::spawn(|| Self::audio_thread(sender));
            ReceiverStream::new(receiver)
        })
    }

    fn audio_thread(sender: Sender<AudioMessage>) {
        let main_loop = MainLoopRc::new(None).unwrap();
        let context = ContextRc::new(&main_loop, None).unwrap();
        let core = context.connect_rc(None).unwrap();
        let registry = core.get_registry_rc().unwrap();

        let sink = RefCell::new(None);
        let sink_listener = RefCell::new(None);

        let registry_clone = RegistryRc::clone(&registry);
        let _registry_listener = registry
            .add_listener_local()
            .global(move |x| {
                Self::handle_global_object(
                    x,
                    sender.clone(),
                    RegistryRc::clone(&registry_clone),
                    &mut sink.borrow_mut(),
                    &mut sink_listener.borrow_mut(),
                )
            })
            .register();

        main_loop.run();
    }

    fn handle_global_object(
        object: &GlobalObject<&DictRef>,
        sender: Sender<AudioMessage>,
        registry: RegistryRc,
        sink: &mut Option<Node>,
        sink_listener: &mut Option<NodeListener>,
    ) {
        if object.type_ != ObjectType::Node
            || object.props.unwrap().get(*keys::MEDIA_CLASS) != Some("Audio/Sink")
            || sink.is_some()
        {
            return;
        }

        let node: Node = registry.bind(object).unwrap();
        let listener = node
            .add_listener_local()
            .param(move |_, _, _, _, pod| Self::handle_props(pod.unwrap(), sender.clone()))
            .register();
        node.subscribe_params(&[ParamType::Props]);

        *sink = Some(node);
        *sink_listener = Some(listener);
    }

    fn handle_props(props: &Pod, sender: Sender<AudioMessage>) {
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
}
