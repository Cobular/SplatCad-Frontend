import {WebAuth, Auth0Callback, Auth0DecodedHash, Auth0ParseHashError} from "auth0-js";
import { get_store_value } from "svelte/internal";
import { user as userStore, isAuthenticated, popupOpen } from "../stores/auth";

let auth0: WebAuth;

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

export async function loginWithPopup(client: WebAuth, options: any): Promise<void> {
  client.authorize(options);
}

export function logout(client: WebAuth) {
  return client.logout({});
}

export function parseHashHandler(auth0: WebAuth): Auth0Callback<Auth0DecodedHash, Auth0ParseHashError> {
  return (err: Auth0ParseHashError, authResult: Auth0DecodedHash) => {
    if (err) {
      console.error(err);
      return;
    }

    console.log(authResult);
    if (authResult && authResult.accessToken) {
      auth0.client.userInfo(authResult.accessToken, (err, user) => {
        if (user) {
          isAuthenticated.set(true);
          console.log(authResult.accessToken)
          userStore.set({
            sub: user.sub,
            authToken: authResult.accessToken
          });
        }
      }
      );
    }
  }
}