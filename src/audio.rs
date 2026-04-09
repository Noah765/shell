use std::{cell::RefCell, thread};

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

use crate::shell::Message;

pub fn audio() -> ReceiverStream<Message> {
    let (sender, receiver) = mpsc::channel(64);
    thread::spawn(|| audio_thread(sender));
    ReceiverStream::new(receiver)
}

fn audio_thread(sender: Sender<Message>) {
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
            handle_global_object(
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
    sender: Sender<Message>,
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
        .param(move |_, _, _, _, pod| handle_props(pod.unwrap(), sender.clone()))
        .register();
    node.subscribe_params(&[ParamType::Props]);

    *sink = Some(node);
    *sink_listener = Some(listener);
}

fn handle_props(props: &Pod, sender: Sender<Message>) {
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

    sender
        .blocking_send(Message::AudioVolumeChanged(volume))
        .unwrap();
}
