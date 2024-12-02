import cls from "classnames";
import React, { useState } from "react";
import { Trans, useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import TwoFactorAuth from "./twoFactorAuth.jsx";
import Header from "../../components/Header.jsx";
import * as Icons from "../../components/icon/index.js";
import { updateProfile } from "../../services/user.js";

export default function () {
  const [isPasswordUpdating, setPasswordUpdating] = useState(false);
  const [isProfileUpdating, setProfileUpdating] = useState(false);
  const dispatch = useDispatch();
  const user = useSelector((state) => state.user);
  const { t } = useTranslation();

  const onProfileUpdate = async function (e) {
    e.preventDefault();

    const name = e.target.name.value;
    const gender = e.target.gender.value;

    if (!name || !gender) {
      return alert(t("nickname and homepage are required"));
    }

    setProfileUpdating(true);
    try {
      await dispatch.user.updateProfile({ name, gender });
    } catch (e) {
      alert(e);
    } finally {
      setProfileUpdating(false);
    }
  };

  const onPasswordUpdate = async function (e) {
    e.preventDefault();

    const password = e.target.password.value;
    const confirm = e.target.confirm.value;

    if (!password || !confirm) {
      return alert(t("please input password"));
    }

    if (password !== confirm) {
      return alert(t("passwords don't match"));
    }

    setPasswordUpdating(true);
    await updateProfile({ password });
    setPasswordUpdating(false);
  };

  const changeAvatar = async function (e) {
    e.preventDefault();

    const url = prompt(t("please input avatar url"));

    if (!url) {
      return;
    }

    await updateProfile({ avatar: url });
    location.reload();
  };

  let baseUrl = window.serverURL;

  if (!baseUrl) {
    const match = location.pathname.match(/(.*?\/)ui/);

    baseUrl = match ? match[1] : "/";
  }
  const qs = new URLSearchParams(location.search);
  let token =
    window.TOKEN || sessionStorage.getItem("TOKEN") || qs.get("token");

  if (!token) {
    token = localStorage.getItem("TOKEN");
  }

  return (
    <>
      <Header />
      <div className="main">
        <div className="body container">
          <div className="typecho-page-title">
            <h2>{t("setting")}</h2>
          </div>
          <div className="row typecho-page-main">
            <div className="col-mb-12 col-tb-3">
              <p>
                <a
                  href="#"
                  title={t("change avatar")}
                  target="_blank"
                  rel="noreferrer"
                  onClick={changeAvatar}
                >
                  <img
                    className="profile-avatar"
                    src={
                      user.avatar ||
                      `https://seccdn.libravatar.org/avatar/${user.mailMd5}?s=220&amp;r=X&amp;d=mm`
                    }
                  />
                </a>
              </p>
              <h2>{user.name}</h2>
              <p>{user.email}</p>
            </div>

            <div
              className="col-mb-12 col-tb-6 col-tb-offset-1 typecho-content-panel"
              role="form"
            >
              <section>
                <h3>{t("profile")}</h3>
                <form method="post" onSubmit={onProfileUpdate}>
                  <ul className="typecho-option">
                    <li>
                      <label className="typecho-label" htmlFor="name">
                        {t("nickname")}
                      </label>
                      <input
                        id="name"
                        name="name"
                        type="text"
                        className="text"
                        defaultValue={user.name}
                      />
                      <p className="description"></p>
                    </li>
                  </ul>

                  <ul className="typecho-option">
                    <li>
                      <label className="typecho-label" htmlFor="url-0-2">
                        {t("gender")}
                      </label>
                      <select
                        name="gender"
                        id="gender"
                        style={{ width: "100%" }}
                        defaultValue={user.gender}
                      >
                        <option value="unknown">{t("unknown")}</option>
                        <option value="male">{t("male")}</option>
                        <option value="female">{t("female")}</option>
                      </select>
                    </li>
                  </ul>

                  <ul className="typecho-option typecho-option-submit">
                    <li>
                      <button
                        type="submit"
                        className="btn primary"
                        disabled={isProfileUpdating}
                      >
                        {t("update my profile")}
                      </button>
                    </li>
                  </ul>
                </form>
              </section>
              <br />
              <section id="social-account">
                <h3>{t("connect to social account")}</h3>
                <div className="account-list">
                  {!window.ALLOW_SOCIALS &&
                    ["wechat", "qq", "weibo", "github", "twitter"/*, "facebook"*/].map(
                      (social) => (
                        <div
                          key={social}
                          className={cls("account-item", social, {
                            bind: user[social],
                          })}
                        >
                          <a
                            href={
                              user[social]
                                ? `https://${social}.com/${user[social]}`
                                : `${baseUrl}oauth/${social}/render?state=${token}`
                            }
                            target={user[social] ? "_blank" : "_self"}
                            rel="noreferrer"
                          >
                            {
                              /* eslint-disable-next-line import-x/namespace */
                              React.createElement(Icons[social])
                            }
                          </a>
                        </div>
                      )
                    )}
                </div>
              </section>
              <br />
              <section id="change-password">
                <h3>{t("change password")}</h3>
                <form method="post" onSubmit={onPasswordUpdate}>
                  <ul className="typecho-option">
                    <li>
                      <label className="typecho-label" htmlFor="password-0-11">
                        {t("password")}
                      </label>
                      <input
                        id="password-0-11"
                        name="password"
                        type="password"
                        className="w-60"
                        autoComplete="new-password"
                      />
                      <p className="description">
                        <Trans i18nKey="password tips"></Trans>
                      </p>
                    </li>
                  </ul>

                  <ul className="typecho-option">
                    <li>
                      <label className="typecho-label" htmlFor="confirm-0-12">
                        {t("password again")}
                      </label>
                      <input
                        id="confirm-0-12"
                        name="confirm"
                        type="password"
                        className="w-60"
                        autoComplete="new-password"
                      />
                      <p className="description">
                        <Trans i18nKey="password again tips"></Trans>
                      </p>
                    </li>
                  </ul>
                  <ul className="typecho-option typecho-option-submit">
                    <li>
                      <button
                        type="submit"
                        className="btn primary"
                        disabled={isPasswordUpdating}
                      >
                        {t("update password")}
                      </button>
                    </li>
                  </ul>
                </form>
              </section>
              <br />
              <TwoFactorAuth />
              <br />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
