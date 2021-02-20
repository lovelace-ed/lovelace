/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

create table if not exists institution (
    id serial primary key,
    "name" text not null unique,
    domain text not null unique,
    created timestamp not null default now(),
    /* settings */
    enforce_same_domain boolean not null
);

create table if not exists administrator (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    institution_id integer not null references institution (id) on delete cascade
);

create table if not exists administrator_invite (
    id serial primary key,
    inviting_user_id integer not null references users (id) on delete cascade,
    invited_user_id integer not null references users (id) on delete cascade,
    institution_id integer not null references institution (id) on delete cascade,
    accepted boolean not null
);

create table if not exists student_group (
    id serial primary key,
    parent_group integer references student_group (id) on delete cascade,
    institution_id integer not null references institution (id) on delete cascade,
    code text not null unique
);

create table if not exists student_group_student (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    student_group_id integer not null references student_group (id) on delete cascade
);

create table if not exists student_group_teacher (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    student_group_id integer not null references student_group (id) on delete cascade
);

create table if not exists student_group_teacher_invite (
    id serial primary key,
    inviting_user_id integer not null references users (id) on delete cascade,
    invited_user_id integer not null references users (id) on delete cascade,
    student_group_id integer not null references student_group (id) on delete cascade,
    accepted boolean not null
);

create table if not exists "class" (
    id serial primary key,
    name text not null,
    description text not null,
    created timestamp not null,
    code text not null unique,
    institution_id integer references institution (id),
    student_group_id integer references student_group (id)
);

create table if not exists class_teacher (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);

create table if not exists class_teacher_invite (
    id serial primary key,
    inviting_user_id integer not null references users (id) on delete cascade,
    invited_user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade,
    accepted boolean not null
);

create table if not exists class_student (
    id serial primary key,
    user_id integer not null references users (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);
