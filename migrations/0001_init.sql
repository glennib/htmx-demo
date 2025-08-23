create table "user"
(
    user_id uuid primary key default uuidv7(),
    name    text not null check ( name <> '' )
);

create table note
(
    note_id    uuid primary key default uuidv7(),
    user_id    uuid      not null references "user" (user_id),
    created_at timestamptz not null default now(),
    updated_at timestamptz,
    title      text        not null default '',
    body       text        not null default '',
    is_done    bool        not null default false
);
