--- 浏览量
create table page_view_counter (
    id serial primary key,
    url varchar(255) not null,
    times int not null default 0,
    reaction0 int not null default 0,
    reaction1 int not null default 0,
    reaction2 int not null default 0,
    reaction3 int not null default 0,
    reaction4 int not null default 0,
    reaction5 int not null default 0,
    reaction6 int not null default 0,
    reaction7 int not null default 0,
    reaction8 int not null default 0,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
--- url字段创建唯一索引，对于频繁根据url查询id，包含id避免回表
create unique index if not exists pvc_uk_url on page_view_counter(url) include (id);
--- 评论状态
create type comment_status as enum('waiting', 'approved', 'spam');
--- 用户评论
create table comments (
    id serial primary key,
    page_id int not null,
    user_id int default null,
    content text not null,
    link varchar(255) default null,
    mail varchar(255) default null,
    nick varchar(255) default null,
    pid int not null default 0,
    rid int not null default 0,
    sticky boolean not null default 'false',
    status comment_status not null,
    star int not null default 0,
    ip varchar(255) not null,
    ua text not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
--- keyset pagination索引，最后包含user_id是为了索引覆盖user_id的过滤，因为user_id的条件有无是动态的
create index if not exists cmt_pgid_pid_created_id_sticky on comments(page_id, pid, created_at, id, sticky, user_id);
create index if not exists cmt_pgid_pid_star_id_sticky on comments(page_id, pid, star, id, sticky, user_id);
--- 用户类型
create type user_type as enum('admin', 'normal');
create type user_gender as enum('unknown', 'male', 'female');
--- 用户
create table users (
    id serial primary key,
    username varchar(40) not null,
    password varchar(255) null,
    email varchar(255) null,
    gender user_gender not null,
    type user_type not null,
    avatar varchar(255) default null,
    mfa boolean not null default 'false',
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);
--- 第三方登录
create table user_oauth(
    id serial primary key,
    user_id int not null,
    provider varchar(50) not null,
    provider_id varchar(255) not null,
    access_token varchar(255) not null,
    refresh_token varchar(255) not null,
    expires_at timestamp not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);