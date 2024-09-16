import request from '../utils/request.js';

export function get2FAToken(email) {
  const query = email ? `?email=${encodeURIComponent(email)}` : '';

  return request({ url: 'token/2fa' + query, method: 'GET' });
}

export function gen2FAToken(data) {
  return request({ url: 'token/2fa', method: 'POST', body: data });
}

export function updateProfile(data) {
  return request({ url: 'user', method: 'PUT', body: data });
}

export function getUserList({ page }) {
  return request({
    url: `user?page=${page}`,
    method: 'GET',
  });
}

export function updateUser({ id, ...data }) {
  return request({ url: `user/${id}`, method: 'PUT', body: data });
}

export function sendRegisterCode(email) {
  return request({ url: `user/register-validate-code`, method: 'POST', body: { email } });
}

export function sendResetCode(email) {
  return request({ url: `user/reset-validate-code`, method: 'POST', body: { email } });
}
