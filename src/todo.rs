use crate::{auth::*, error_template::ErrorTemplate, ui::{CenteredCard, Form, FormCheckbox, FormInput}};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

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

// The struct name and path prefix arguments are optional.
#[server]
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
                    <A href="/">
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
        <div>
            <MultiActionForm action=add_todo>
                <label>"Add a Todo" <input type="text" name="title"/></label>
                <input type="submit" value="Add"/>
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
                                                                {todo.title} ": Created at " {todo.created_at} " by "
                                                                {todo.user.unwrap_or_default().username}
                                                                <ActionForm action=delete_todo>
                                                                    <input type="hidden" name="id" value=todo.id/>
                                                                    <input type="submit" value="X" class="btn"/>
                                                                </ActionForm>
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
                                        <li class="pending">
                                            {move || submission.input.get().map(|data| data.title)}
                                        </li>
                                    }
                                })
                                .collect_view()
                        };
                        view! { <ul>{existing_todos} {pending_todos}</ul> }
                    }}

                </ErrorBoundary>
            </Transition>
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
                    id="username"
                    label="Username"
                    placeholder="Username"
                    maxlength=32
                />
                <FormInput
                    input_type="password"
                    id="password"
                    label="Password"
                    placeholder="Password"
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
                    id="username"
                    label="Username"
                    placeholder="Username"
                    maxlength=32
                />
                <FormInput
                    input_type="password"
                    id="password"
                    label="Password"
                    placeholder="Password"
                />
                <FormInput
                    input_type="password"
                    id="password_confirmation"
                    label="Confirm Password"
                    placeholder="Password again"
                />
                <FormCheckbox label="Remember me?" id="remember"/>
            </Form>
        </CenteredCard>
    }
}
