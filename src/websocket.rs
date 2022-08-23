use crate::data::event::{OneBotEvent, OneBotTypedEvent};
use crate::data::message::OneBotMessageEvent;
use atri_plugin::event::Event;
use atri_plugin::listener::{Listener, ListenerGuard};
use std::sync::Arc;

pub fn ws_listener(tx: tokio::sync::broadcast::Sender<Arc<OneBotEvent>>) -> ListenerGuard {
    Listener::listening_on_always(move |e: Event| {
        let tx = tx.clone();
        async move {
            match e {
                Event::GroupMessageEvent(e) => {
                    let ob = OneBotEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        time: 0.0,
                        inner: OneBotTypedEvent::Message(OneBotMessageEvent::Group {
                            message: e.message().into(),
                            group_id: e.group().id().to_string(),
                        }),
                        sub_type: "".to_string(),
                        bot_self: Some(e.bot().into()),
                    };

                    let arc = Arc::new(ob);

                    let _ = tx.send(arc);
                }
                _ => {}
            }
        }
    })
}
