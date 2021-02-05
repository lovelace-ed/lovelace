-- Your SQL goes here

create table if not exists calendar (
    id serial primary key,
    calendar_type integer not null,
    user_id integer not null references users (id) on delete cascade
);

create table if not exists google_calendar (
    id serial primary key,
    calendar_id integer not null references calendar (id) on delete cascade,
    refresh_token text not null,
    access_token text not null,
    lovelace_calendar_id text not null
);

create table if not exists caldav (
    id serial primary key,
    calendar_id integer not null references calendar (id) on delete cascade,
    username text not null,
    "password" text not null,
    "url" text not null
);

create table if not exists caldav_unauthenticated (
    id serial primary key,
    calendar_id integer not null references calendar (id) on delete cascade,
    "url" text not null
);
