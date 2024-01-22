import * as Types from "./operations";

import { GraphQLClient } from "graphql-request";
import * as Dom from "graphql-request/dist/types.dom";

export const GetTokens = `
    query GetTokens($owner: String) {
  current_token_ownerships_v2(
    where: {current_token_data: {current_collection: {collection_name: {_eq: "Aptos Summits"}}}, owner_address: {_eq: $owner}}
  ) {
    current_token_data {
      token_data_id
    }
  }
}
    `;

export type SdkFunctionWrapper = <T>(
  action: (requestHeaders?: Record<string, string>) => Promise<T>,
  operationName: string,
  operationType?: string,
) => Promise<T>;

const defaultWrapper: SdkFunctionWrapper = (
  action,
  _operationName,
  _operationType,
) => action();

export function getSdk(
  client: GraphQLClient,
  withWrapper: SdkFunctionWrapper = defaultWrapper,
) {
  return {
    GetTokens(
      variables?: Types.GetTokensQueryVariables,
      requestHeaders?: Dom.RequestInit["headers"],
    ): Promise<Types.GetTokensQuery> {
      return withWrapper(
        (wrappedRequestHeaders) =>
          client.request<Types.GetTokensQuery>(GetTokens, variables, {
            ...requestHeaders,
            ...wrappedRequestHeaders,
          }),
        "GetTokens",
        "query",
      );
    },
  };
}
export type Sdk = ReturnType<typeof getSdk>;
