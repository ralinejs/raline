import request from '../utils/request.js';

export async function getUserInfo() {
  return request('token').catch(() => {
    logout();
    Promise.reject(new Error('get userinfo failed'));
  });
}

export async function login({ email, password, code, recaptchaV3, turnstile }) {
  return request({
    url: 'token',
    method: 'POST',
    body: { email, password, code, recaptchaV3, turnstile },
  });
}

export async function logout() {
  window.TOKEN = null;
  sessionStorage.removeItem('TOKEN');
  localStorage.removeItem('TOKEN');
}

export async function register(user) {
  return request({ url: 'user', method: 'POST', body: user });
}

export async function forgot({ email, password, validate_code }) {
  return request({
    url: 'user/password',
    method: 'POST',
    body: { email, password, validate_code },
  });
}
