import React, { useEffect, useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { Link, useNavigate } from "react-router-dom";

import Header from "../../components/Header.jsx";
import { sendResetCode } from "../../services/user.js";

export default function () {
  const { t } = useTranslation();
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const formRef = useRef();
  const user = useSelector((state) => state.user);
  const [error, setError] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (user && user.email) {
      navigate("/ui", { replace: true });
    }
  }, [navigate]);

  const onSendCode = async function (e) {
    let email = formRef.current.email.value;
    if (!email) {
      return setError(t("please input email"));
    }
    await sendResetCode(email);
    let timeout = 60;
    function stopwatch() {
      if (timeout <= 0) {
        e.target.value = t("send code");
        e.target.disabled = false;
      } else {
        e.target.value = `${timeout}s`;
        e.target.disabled = true;
        timeout--;
        setTimeout(stopwatch, 1000);
      }
    }
    stopwatch();
  };

  const onSubmit = async function (e) {
    e.preventDefault();
    setError(false);

    const email = e.target.email.value;
    if (!email) {
      return setError(t("please input email"));
    }
    const code = e.target.code.value;
    if (!code) {
      return setError(t("minimum 6 characters required"));
    }

    const password = e.target.password.value;
    const passwordAgain = e.target["password-again"].value;

    if (!password || !passwordAgain || passwordAgain !== password) {
      return setError(t("passwords don't match"));
    }
    try {
      setSubmitting(true);
      await dispatch.user.forgot({
        email,
        validate_code: code,
        password,
      });
      alert(t("find password success! please go to your mailbox to reset it!"));
      navigate("/ui/login");
    } catch (e) {
      setError(e.message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <>
      <Header />
      <div
        className="message popup notice"
        style={{
          position: "fixed",
          top: 0,
          display: error ? "block" : "none",
        }}
      >
        <ul>{error ? <li>{error}</li> : null}</ul>
      </div>
      <div className="typecho-login-wrap">
        <div className="typecho-login">
          <form
            method="post"
            name="login"
            role="form"
            ref={formRef}
            onSubmit={onSubmit}
          >
            <p>
              <label htmlFor="email" className="sr-only">
                {t("email")}
              </label>
              <input
                type="text"
                id="email"
                name="email"
                placeholder={t("email")}
                className="text-l w-100"
              />
            </p>
            <p className="input-group">
              <label htmlFor="code" className="sr-only">
                {t("verification code")}
              </label>
              <input
                type="text"
                id="code"
                name="code"
                placeholder={t("verification code")}
                className="text-l w-60"
              />
              <input
                type="button"
                className="btn btn-l w-40 primary"
                value={t("send code")}
                onClick={onSendCode}
              />
            </p>
            <p>
              <label htmlFor="password" className="sr-only">
                {t("new password")}
              </label>
              <input
                type="password"
                id="password"
                name="password"
                className="text-l w-100"
                placeholder={t("new password")}
              />
            </p>
            <p>
              <label htmlFor="password-again" className="sr-only">
                {t("password again")}
              </label>
              <input
                type="password"
                id="password-again"
                name="password-again"
                className="text-l w-100"
                placeholder={t("password again")}
              />
            </p>
            <p className="submit">
              <button
                type="submit"
                disabled={submitting}
                className="btn btn-l w-100 primary"
              >
                {t("reset password")}
              </button>
            </p>
          </form>

          <p className="more-link">
            <Link to="/ui">{t("back to home")}</Link> •{" "}
            <Link to="/ui/login">{t("register.login")}</Link>
          </p>
        </div>
      </div>
    </>
  );
}
