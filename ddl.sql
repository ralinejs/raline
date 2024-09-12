--- 评论状态
create type comment_status as enum('waiting', 'approved', 'spam');
--- 用户评论
create sequence comments_seq;
create table comments (
    id bigint not null default nextval ('comments_seq'),
    user_id bigint default null,
    content text,
    ip inet not null,
    link varchar(255) default null,
    mail varchar(255) default null,
    nick varchar(255) default null,
    pid bigint default null,
    rid bigint default null,
    sticky boolean not null default 'false',
    status comment_status not null,
    star int default null,
    ua text,
    url varchar(255) default null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (id)
);
--- 浏览量
create sequence view_counter_seq;
create table view_counter (
    id bigint not null default nextval ('view_counter_seq'),
    times int not null default 0,
    url varchar(255) not null default '',
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (id)
);
--- 用户类型
create type user_type as enum('admin', 'guest', 'normal');
create type user_gender as enum('unknown', 'male', 'female');
--- 用户
create sequence users_seq;
create table users (
    id bigint not null default nextval ('users_seq'),
    username varchar(40) not null,
    password varchar(255) null,
    email varchar(255) null,
    gender user_gender not null,
    type user_type not null,
    url varchar(255) default null,
    avatar varchar(255) default null,
    mfa boolean not null default 'false',
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (id)
);
--- 第三方登录
create sequence user_oauth_seq;
create table user_oauth(
    id bigint not null default nextval ('user_oauth_seq'),
    user_id bigint not null,
    provider varchar(50) not null,
    provider_id varchar(255) not null,
    access_token varchar(255) not null,
    refresh_token varchar(255) not null,
    expires_at timestamp not null,
    created_at timestamp not null,
    primary key (id)
);