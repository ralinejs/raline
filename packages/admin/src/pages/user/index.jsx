import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import Header from "../../components/Header.jsx";
import Paginator from "../../components/Paginator.jsx";
import { getUserList, updateUser } from "../../services/user.js";
import { buildAvatar } from "../manage-comments/utils.js";

export default function () {
  const currentUser = useSelector((state) => state.user);
  const { t } = useTranslation();
  const [list, setList] = useState({
    page: 1,
    total_pages: 0,
    content: [],
  });

  useEffect(() => {
    getUserList({ page: list.page }).then((data) => {
      console.log(data);
      setList({ ...list, ...data });
    });
  }, [list.page]);

  const createActions = (user) =>
    [
      {
        key: "admin",
        name: t("set administrator"),
        show: user.type === "guest",
        async action(e) {
          e.preventDefault();

          await updateUser({
            id: user.objectId,
            type: "admin",
          });
          user.type = "admin";
          setList({ ...list });
        },
      },
      {
        key: "guest",
        name: t("set guest"),
        show: user.type === "admin",
        async action(e) {
          e.preventDefault();
          if (user.objectId === currentUser.objectId) {
            return alert(t("You can't set yourself to be guest!"));
          }

          await updateUser({
            id: user.objectId,
            type: "guest",
          });
          user.type = "guest";
          setList({ ...list });
        },
      },
      {
        key: "label",
        name: t("set label"),
        show: true,
        async action(e) {
          e.preventDefault();

          const label = prompt(t("please enter an exclusive label"));

          await updateUser({
            id: user.objectId,
            label,
          });
          user.label = label;
          setList({ ...list });
        },
      },
      // todo
      // {
      //   key: 'delete',
      //   name: t('delete'),
      //   show: true,
      //   async action() {},
      // },
    ].filter(({ show }) => show);

  const getRole = (type) => {
    if (/^verify/.test(type)) {
      return t("verify");
    }

    return t(type);
  };

  return (
    <>
      <Header />
      <div className="main">
        <div className="body container">
          <div className="typecho-page-title">
            <h2>{t("manage users")}</h2>
          </div>
          <div className="row typecho-page-main" role="main">
            <div className="col-mb-12 typecho-list">
              <form
                method="post"
                name="manage_comments"
                className="operate-form"
              >
                <div className="typecho-table-wrap">
                  <table className="typecho-list-table">
                    <colgroup>
                      <col width="6%" />
                      <col width="15%" />
                      <col width="20%" />
                      <col width="15%" />
                      <col width="20%" />
                    </colgroup>
                    <thead>
                      <tr>
                        <th> </th>
                        <th>{t("nickname")}</th>
                        <th>{t("email")}</th>
                        <th>{t("role")}</th>
                        <th>{t("exclusive label")}</th>
                        <th>{t("action")}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {list.content.map((user) => (
                        <tr id={`user-${user.id}`} key={user.id}>
                          <td style={{ verticalAlign: "top" }}>
                            <div className="user-avatar">
                              <img
                                className="avatar"
                                src={buildAvatar(user.email, user.avatar)}
                                alt={user.username}
                                width="40"
                                height="40"
                              />
                            </div>
                          </td>
                          <td>
                            {/* <a
                              href={
                                user.url
                                  ? "#"
                                  : !/^https:\/\//.test(user.url)
                                  ? "https://" + user.url
                                  : user.url
                              }
                              rel="external nofollow noreferrer"
                              target="_blank"
                            > */}
                              {user.username}
                            {/* </a> */}
                          </td>
                          <td>
                            <a
                              href={`mailto:${user.email}`}
                              target="_blank"
                              rel="noreferrer"
                            >
                              {user.email}
                            </a>
                          </td>
                          <td>{getRole(user.type)}</td>
                          <td>{user.label}</td>
                          <td className="comment-action">
                            {createActions(user).map(
                              ({ key, disable, name, action }) => {
                                return disable ? (
                                  <span className="weak" key={key}>
                                    {name}
                                  </span>
                                ) : (
                                  <a
                                    key={key}
                                    href="#"
                                    className={`operate-${key}`}
                                    onClick={action}
                                  >
                                    {name}
                                  </a>
                                );
                              }
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </form>

              <div className="typecho-list-operate clear-fix">
                <form method="get">
                  <Paginator
                    current={list.page}
                    total={list.total_pages}
                    onChange={(page) => setList({ ...list, page })}
                  />
                </form>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
