use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc::Sender, Arc, Mutex},
    task::{Context, Poll}, thread::{self, sleep}, time::Duration
};
use crate::sync;
use common::Request;

#[derive(Debug)]
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
    type Output = Request;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        state.waker = Some(cx.waker().clone());

        match &state.data {
            Some(d) => Poll::Ready(d.clone()),
            None => {
                match self.sync_tx.send(sync::Message::Request(
                    self.request.request_id.clone(),
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
