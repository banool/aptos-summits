import { useQuery } from "@tanstack/react-query";
import { useGlobalState } from "../context/GlobalState";
import { AccountAddressInput } from "@aptos-labs/ts-sdk";
import { GetTokensQuery } from "../codegen/indexer/generated/operations";
import { GetTokens } from "../codegen/indexer/generated/queries";

export function useTokens(
  userAddress: AccountAddressInput,
  { enabled }: { enabled?: boolean },
) {
  const [globalState] = useGlobalState();
  return useQuery<string[]>({
    queryKey: ["getGames", userAddress],
    queryFn: async () => {
      const response = await globalState.client.queryIndexer<GetTokensQuery>({
        query: {
          query: GetTokens,
          variables: { owner: userAddress },
        },
      });
      return response.current_token_ownerships_v2.map(
        (data) => data.current_token_data!.token_data_id,
      );
    },
    refetchInterval: 5000,
    retry: true,
    enabled,
  });
}
