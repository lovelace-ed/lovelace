/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

/* A list of "synchronous" tasks (e.g. lessons) that happen in a set space of time. */
create table if not exists class_synchronous_task (
    id serial primary key,
    title text not null,
    description text not null,
    created timestamp not null default now(),
    start_time timestamp not null,
    end_time timestamp not null,
    class_teacher_id integer not null references class_teacher (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);

/* A synchronous task which has been issued to a student. */
create table if not exists student_class_synchronous_task (
    id serial primary key,
    class_student_id integer not null references class_student (id) on delete cascade,
    class_synchronous_task_id integer not null references class_synchronous_task (id) on delete cascade
);

/* A list of "asynchronous" tasks (e.g. homework, coursework) which needs to happen by a specific
point in time.*/
create table if not exists class_asynchronous_task (
    id serial primary key,
    title text not null,
    description text not null,
    created timestamp not null default now(),
    due_date timestamp not null,
    class_teacher_id integer not null references class_teacher (id) on delete cascade,
    class_id integer not null references class (id) on delete cascade
);

/* An asynchronous task which has been issued to a student. */
create table if not exists student_class_asynchronous_task (
    id serial primary key,
    class_student_id integer not null references class_student (id) on delete cascade,
    class_asynchronous_task_id integer not null references class_asynchronous_task (id) on delete cascade,
    completed boolean not null
);
