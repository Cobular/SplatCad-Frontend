import type { Auth0Callback, Auth0DecodedHash, Auth0ParseHashError, WebAuth } from "auth0-js";
import { isAuthenticated, user as userStore } from "../stores/auth";

export async function loginWithPopup(client: WebAuth, options: any): Promise<void> {
  client.authorize(options);
}

export function logout(client: WebAuth) {
  return client.logout({});
}

export function parseHashHandler(auth0: WebAuth): Auth0Callback<Auth0DecodedHash, Auth0ParseHashError> {
  return (err: Auth0ParseHashError, authResult: Auth0DecodedHash) => {
    if (err === null && authResult === null) {
      return
    }

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
            authToken: authResult.accessToken,
            expireTime: new Date().getTime() + authResult.expiresIn * 1000
          });
        }
      }
      );
    }
  }
}