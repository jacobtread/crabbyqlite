use gpui::App;

/// Handles resolving the future calling the callback with the future output
/// and the cx
pub fn resolve_async_callback_cx<O, F, C>(cx: &mut App, future: F, callback: C)
where
    F: Future<Output = O> + 'static,
    C: FnOnce(&mut App, O) + 'static,
{
    cx.spawn(async move |cx| {
        let output = future.await;

        cx.update(move |cx| {
            callback(cx, output);
        });
    })
    .detach();
}
