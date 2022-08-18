import {WebAuth} from "auth0-js";

let auth0: WebAuth | undefined = undefined;

export function createClient(): WebAuth {
  if (auth0 === undefined) {
    auth0 = new WebAuth({
      clientID: '3CKz6QuRKeiPk9AZh6wS5NVc9Og5KRGH',
      domain: 'dev-kcgkylif.us.auth0.com',
      responseType: 'token id_token',
      audience: "https://splatcad-api.cobular.com",
      redirectUri: window.location.origin + "/#/callback",
      scope: 'openid profile read:self modify:self'
    });
  }
  return auth0;
}