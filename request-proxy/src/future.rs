use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc::Sender, Arc, Mutex},
    task::{Context, Poll},
};

use crate::{sync, common::{Response, Request}};

pub struct ReqResFuture {
    pub sync_tx: Sender<sync::Message>,
    pub request: Request,
    pub state: sync::FutureHandle,
}

impl ReqResFuture {
    pub fn new(request: Request, sync_tx: Sender<sync::Message>) -> Self {
        ReqResFuture {
            sync_tx,
            request,
            state: Arc::new(Mutex::new(sync::FutureHandleData {
                data: None,
                waker: None,
            })),
        }
    }
}

impl Future for ReqResFuture {
    type Output = Response;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        match &state.data {
            Some(d) => Poll::Ready(d.clone()),
            None => {
                state.waker = Some(cx.waker().clone());
                match self.sync_tx.send(sync::Message::Request(
                    Arc::clone(&self.state),
                    self.request.clone(),
                )) {
                    Ok(_) => {},
                    Err(e) => eprintln!("Error sending message from Future: {e}"),
                };
                Poll::Pending
            }
        }
    }
}
