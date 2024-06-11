use crate::{auth::*, error_template::ErrorTemplate, ui::{ActionIcon, CenteredCard, Container, Form, FormCheckbox, FormInput}};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use icondata as i;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    id: u32,
    user: Option<User>,
    title: String,
    created_at: String,
    completed: bool,
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::Todo;
    use crate::auth::{ssr::AuthSession, User};
    use leptos::*;
    use sqlx::SqlitePool;

    pub fn pool() -> Result<SqlitePool, ServerFnError> {
        use_context::<SqlitePool>()
            .ok_or_else(|| ServerFnError::ServerError("Pool missing.".into()))
    }

    pub fn auth() -> Result<AuthSession, ServerFnError> {
        use_context::<AuthSession>().ok_or_else(|| {
            ServerFnError::ServerError("Auth session missing.".into())
        })
    }

    #[derive(sqlx::FromRow, Clone)]
    pub struct SqlTodo {
        id: u32,
        user_id: i64,
        title: String,
        created_at: String,
        completed: bool,
    }

    impl SqlTodo {
        pub async fn into_todo(self, pool: &SqlitePool) -> Todo {
            Todo {
                id: self.id,
                user: User::get(self.user_id, pool).await,
                title: self.title,
                created_at: self.created_at,
                completed: self.completed,
            }
        }
    }
}

#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<Todo>, ServerFnError> {
    use self::ssr::{pool, SqlTodo};
    use futures::future::join_all;

    let pool = pool()?;

    Ok(join_all(
        sqlx::query_as::<_, SqlTodo>("SELECT * FROM todos")
            .fetch_all(&pool)
            .await?
            .iter()
            .map(|todo: &SqlTodo| todo.clone().into_todo(&pool)),
    )
    .await)
}

#[server(AddTodo, "/api")]
pub async fn add_todo(title: String) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let user = get_user().await?;
    let pool = pool()?;

    let id = match user {
        Some(user) => user.id,
        None => -1,
    };

    // Fake API delay
    std::thread::sleep(std::time::Duration::from_millis(1250));

    Ok(sqlx::query(
        "INSERT INTO todos (title, user_id, completed) VALUES (?, ?, false)",
    )
    .bind(title)
    .bind(id)
    .execute(&pool)
    .await
    .map(|_| ())?)
}

#[server(UpdateTodo, "/api")]
pub async fn update_todo(id: u32, completed: bool) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = pool()?;

    Ok(sqlx::query(
        "UPDATE todos SET completed = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(completed)
    .execute(&pool)
    .await
    .map(|_| ())?)
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo(id: u16) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = pool()?;

    Ok(sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map(|_| ())?)
}

#[component]
pub fn TodoApp() -> impl IntoView {
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();
    let signup = create_server_action::<Signup>();

    let user = create_resource(
        move || {
            (
                login.version().get(),
                signup.version().get(),
                logout.version().get(),
            )
        },
        move |_| get_user(),
    );
    provide_meta_context();

    view! {
        <Title text="Todo App"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="leptos" href="/pkg/kreqo-habits.css"/>
        <Html lang="en" class="h-full bg-base-200"/>
        <Body class="h-full flex flex-col"/>
        <Router>
            <header class="navbar bg-base-100 px-6">
                <div class="flex-1">
                    <A href="/" class="btn btn-ghost">
                        <h1 class="text-2xl font-bold text-primary">"My Tasks"</h1>
                    </A>
                </div>
                <div class="flex-none">
                    <Transition fallback=move || {
                        view! { <span class="loading loading-spinner"></span> }
                    }>
                        {move || {
                            let login_section = move || {
                                view! {
                                    <A href="/signup" class="btn btn-ghost text-lg">
                                        "Sign up"
                                    </A>
                                    <A href="/login" class="btn btn-ghost text-lg">
                                        "Log in"
                                    </A>
                                }
                                    .into_view()
                            };
                            user.get()
                                .map(|user| match user {
                                    Err(e) => {
                                        view! {
                                            login_section()
                                            <span>{format!("Login error: {}", e)}</span>
                                        }
                                            .into_view()
                                    }
                                    Ok(None) => login_section(),
                                    Ok(Some(user)) => {
                                        view! {
                                            <div class="dropdown relative">
                                                <div
                                                    tabindex="0"
                                                    role="button"
                                                    class="btn btn-ghost text-lg"
                                                >
                                                    {user.username}
                                                </div>
                                                <ul
                                                    tabindex="0"
                                                    class="dropdown-content z-[1] menu relative right-0 mt-1 p-2 w-52 bg-base-200 border border-neutral rounded-xl"
                                                >
                                                    <li>
                                                        <a class="btn btn-ghost text-lg">"Settings"</a>
                                                    </li>
                                                    <li>
                                                        <a
                                                            on:click=move |_| {
                                                                logout.dispatch(Logout {});
                                                            }

                                                            class="btn btn-ghost text-lg"
                                                        >
                                                            "Log out"
                                                        </a>
                                                    </li>
                                                </ul>
                                            </div>
                                        }
                                            .into_view()
                                    }
                                })
                        }}

                    </Transition>
                </div>
            </header>
            <main class="flex-1">
                <Routes>
                    <Route path="" view=Todos/>
                    <Route path="signup" view=move || view! { <Signup action=signup/> }/>
                    <Route path="login" view=move || view! { <Login action=login/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Todos() -> impl IntoView {
    let add_todo = create_server_multi_action::<AddTodo>();
    let delete_todo = create_server_action::<DeleteTodo>();
    let submissions = add_todo.submissions();

    // List of todos is loaded from the server in reaction to changes
    let todos = create_resource(
        move || (add_todo.version().get(), delete_todo.version().get()),
        move |_| get_todos(),
    );

    view! {
        <Container>
            <MultiActionForm action=add_todo class="flex items-center gap-4 mb-4">
                <label class="input input-bordered flex items-center flex-1 text-xl gap-4">
                    <span class="text-primary">"Todo Title"</span>
                    <input type="text" name="title"/>
                </label>
                <button type="submit" class="btn btn-primary text-lg">
                    "Add Todo"
                </button>
            </MultiActionForm>
            <Transition fallback=move || view! { <p>"Loading..."</p> }>
                <ErrorBoundary fallback=|errors| {
                    view! { <ErrorTemplate errors=errors/> }
                }>
                    {move || {
                        let existing_todos = {
                            move || {
                                todos
                                    .get()
                                    .map(move |todos| match todos {
                                        Err(e) => {
                                            view! {
                                                <pre class="error">"Server Error: " {e.to_string()}</pre>
                                            }
                                                .into_view()
                                        }
                                        Ok(todos) => {
                                            if todos.is_empty() {
                                                view! { <p>"No tasks were found."</p> }.into_view()
                                            } else {
                                                todos
                                                    .into_iter()
                                                    .map(move |todo| {
                                                        view! {
                                                            <li>
                                                                <Todo todo delete_todo/>
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()
                                            }
                                        }
                                    })
                                    .unwrap_or_default()
                            }
                        };
                        let pending_todos = move || {
                            submissions
                                .get()
                                .into_iter()
                                .filter(|submission| submission.pending().get())
                                .map(|submission| {
                                    view! {
                                        <li>
                                            <PendingTodo input=submission.input/>
                                        </li>
                                    }
                                })
                                .collect_view()
                        };
                        view! {
                            <div class="h-full">
                                <ul class="overflow-auto space-y-2">
                                    {existing_todos} {pending_todos}
                                </ul>
                            </div>
                        }
                    }}

                </ErrorBoundary>
            </Transition>
        </Container>
    }
}

#[component]
pub fn PendingTodo(input: RwSignal<Option<AddTodo>>) -> impl IntoView {
    view! {
        <div class="flex gap-2 animate-pulse">
            <div class="h-12 flex flex-1 items-center gap-4 px-3 bg-base-100 rounded-xl">
                <input type="checkbox" class="checkbox checkbox-accent" disabled/>
                <span class="text-xl">{input.get().map(|data| data.title)}</span>
                <span class="flex-1 text-right text-xl">"Loading..."</span>
            </div>
        </div>
    }
}

#[component]
pub fn Todo(todo: Todo, delete_todo: Action<DeleteTodo, Result<(), ServerFnError>>) -> impl IntoView {
    let (completed, set_completed) = create_signal(todo.completed);

    view! {
        <div class="flex gap-2">
            <div class="h-12 flex flex-1 items-center gap-4 px-3 bg-base-100 rounded-xl">
                <input
                    type="checkbox"
                    class="checkbox checkbox-accent"
                    checked=completed
                    on:change=move |ev| {
                        let checked = event_target_checked(&ev);
                        set_completed.set(checked);
                        spawn_local(async move {
                            update_todo(todo.id, checked).await.unwrap();
                        });
                    }
                />

                <span class="text-xl">{todo.title}</span>
                <span class="flex-1 text-right">
                    "Created at " <span class="text-primary">{todo.created_at}</span> " by "
                    <span class="text-primary">{todo.user.unwrap_or_default().username}</span>
                </span>
            </div>
            <ActionIcon
                action=delete_todo
                icon=i::LuTrash2
                class="btn-ghost bg-base-100 text-error rounded-xl"
            >
                <input type="hidden" name="id" value=todo.id/>
            </ActionIcon>
        </div>
    }
}

#[component]
pub fn Login(
    action: Action<Login, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <CenteredCard>
            <Form action title="Connect to Your Account" submit="Log In">
                <FormInput
                    input_type="text"
                    name="username"
                    label="Username"
                    placeholder="username"
                    maxlength=32
                />
                <FormInput
                    input_type="password"
                    name="password"
                    label="Password"
                    placeholder="password"
                />
                <FormCheckbox label="Remember me?" id="remember"/>
            </Form>
        </CenteredCard>
    }
}

#[component]
pub fn Signup(
    action: Action<Signup, Result<(), ServerFnError>>,
) -> impl IntoView {
    view! {
        <CenteredCard>
            <Form action title="Create Your Account" submit="Sign Up">
                <FormInput
                    input_type="text"
                    name="username"
                    label="Username"
                    placeholder="username"
                    maxlength=32
                />
                <FormInput
                    input_type="password"
                    name="password"
                    label="Password"
                    placeholder="password"
                />
                <FormInput
                    input_type="password"
                    name="password_confirmation"
                    label="Confirm Password"
                    placeholder="password again"
                />
                <FormCheckbox label="Remember me?" id="remember"/>
            </Form>
        </CenteredCard>
    }
}
