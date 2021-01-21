-- Your SQL goes here
create table if not exists class_message (
  id serial primary key,
  title text not null,
  contents text not null,
  created_at timestamp not null default now(),
  /* The user who created this. We might want to change the delete handling here to be something
  other than "on delete cascade" */
  user_id integer not null references users (id) on delete cascade,
  /* The class that this message has been sent to. */
  class_id integer not null references class (id) on delete cascade,
  /* Whether or not the message has been edited – in the future we might want to store an edit
  history (including who made what edits). */
  edited boolean not null default 'false'
);

create table if not exists class_message_reply (
    id serial primary key,
    contents text not null,
    created_at timestamp not null default now(),
    /* Whether or not the message has been edited – in the future we might want to store an edit
    history (including who made what edits). */
    edited boolean not null default 'f',
    /* The user who created this. We might want to change the delete handling here to be something
    other than "on delete cascade" */
    user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade,
    class_message_id integer not null references class_message (id) on delete cascade
);
