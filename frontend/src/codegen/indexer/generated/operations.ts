import * as Types from "./types";

export type GetTokensQueryVariables = Types.Exact<{
  owner?: Types.InputMaybe<Types.Scalars["String"]>;
}>;

export type GetTokensQuery = {
  __typename?: "query_root";
  current_token_ownerships_v2: Array<{
    __typename?: "current_token_ownerships_v2";
    current_token_data?: {
      __typename?: "current_token_datas_v2";
      token_data_id: string;
    } | null;
  }>;
};
