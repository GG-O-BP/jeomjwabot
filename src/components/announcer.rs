use leptos::prelude::*;

/// 페이지 1개의 전역 라이브 영역 슬롯.
///
/// WebAIM A-3: 다중 라이브 영역은 사용자에게 부담을 준다. 이 컴포넌트는
/// polite + assertive 슬롯을 각 1개씩만 두고, 자식 컴포넌트는 context로
/// 받은 핸들로만 발화한다. 빈 메시지는 P 자체를 미마운트하여 잡음 제거.
#[derive(Clone, Copy)]
pub struct AnnouncerHandle {
    polite: RwSignal<Option<String>>,
    #[allow(dead_code)] // 향후 권한 회수 등 긴급 알림에 사용 예정
    assertive: RwSignal<Option<String>>,
}

impl AnnouncerHandle {
    pub fn polite(self, msg: impl Into<String>) {
        self.polite.set(Some(msg.into()));
    }
    #[allow(dead_code)]
    pub fn assertive(self, msg: impl Into<String>) {
        self.assertive.set(Some(msg.into()));
    }
}

#[component]
pub fn LiveAnnouncer() -> impl IntoView {
    let polite: RwSignal<Option<String>> = RwSignal::new(None);
    let assertive: RwSignal<Option<String>> = RwSignal::new(None);

    provide_context(AnnouncerHandle { polite, assertive });

    view! {
        <div class="sr-only">
            <Show when=move || polite.with(|m| m.as_deref().is_some_and(|s| !s.is_empty()))>
                <p role="status" aria-live="polite" aria-atomic="true">
                    {move || polite.get().unwrap_or_default()}
                </p>
            </Show>
            <Show when=move || assertive.with(|m| m.as_deref().is_some_and(|s| !s.is_empty()))>
                <p role="alert" aria-live="assertive" aria-atomic="true">
                    {move || assertive.get().unwrap_or_default()}
                </p>
            </Show>
        </div>
    }
}
