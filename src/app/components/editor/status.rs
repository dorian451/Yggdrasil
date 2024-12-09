use leptos::{either::EitherOf3, prelude::*};
use tracing::info;

/// Levels of severity the [StatusIndicator] can show
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusLevel {
    Good,
    Warn,
    Bad,
}

/// Displays an icon (check mark, warning sign, or cross) to represent the status of something
#[component]
pub fn StatusIndicator(state: Signal<StatusLevel>) -> impl IntoView {
    view! {
        {move || match state.get() {
            StatusLevel::Warn => {
                EitherOf3::A(
                    view! {
                        <svg
                            class="h-full text-black aspect-square size-6"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="yellow"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
                            />
                        </svg>
                    },
                )
            }
            StatusLevel::Bad => {
                EitherOf3::B(
                    view! {
                        <svg
                            class="h-full text-red-600 aspect-square size-6"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="3"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M6 18 18 6M6 6l12 12"
                            />
                        </svg>
                    },
                )
            }
            StatusLevel::Good => {
                EitherOf3::C(
                    view! {
                        <svg
                            class="h-full text-green-500 aspect-square size-6"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="3"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="m4.5 12.75 6 6 9-13.5"
                            />
                        </svg>
                    },
                )
            }
        }}
    }
}
