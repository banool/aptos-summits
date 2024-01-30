import { Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk";
import React, { createContext, useMemo } from "react";
import { defaultNetwork } from "../constants";
import { useNetworkSelector } from "./networkSelection";

export type GlobalState = {
  /** derived from external state ?network=<network> query parameter - e.g. devnet */
  readonly network: Network;
  /** derived from network_value */
  readonly client: Aptos;
  /** derived from network_value */
  readonly moduleAddress: string;
};

type GlobalActions = {
  selectNetwork: ReturnType<typeof useNetworkSelector>[1];
};

function deriveGlobalState({ network }: { network: Network }): GlobalState {
  const config = new AptosConfig({ network });
  const client = new Aptos(config);
  const { moduleAddress } = getModuleAddressAndName(network);
  return {
    network,
    client,
    moduleAddress,
  };
}

const initialGlobalState = deriveGlobalState({
  network: defaultNetwork,
});

export const GlobalStateContext = createContext(initialGlobalState);
export const GlobalActionsContext = createContext({} as GlobalActions);

export const GlobalStateProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const [selectedNetwork, selectNetwork] = useNetworkSelector();
  const globalState: GlobalState = useMemo(
    () =>
      deriveGlobalState({
        network: selectedNetwork,
      }),
    [selectedNetwork],
  );

  const globalActions = useMemo(
    () => ({
      selectNetwork,
    }),
    [selectNetwork],
  );

  return (
    <GlobalStateContext.Provider value={globalState}>
      <GlobalActionsContext.Provider value={globalActions}>
        {children}
      </GlobalActionsContext.Provider>
    </GlobalStateContext.Provider>
  );
};

export const useGlobalState = () =>
  [
    React.useContext(GlobalStateContext),
    React.useContext(GlobalActionsContext),
  ] as const;

export const getModuleIdentifier = (
  state: GlobalState,
  moduleName: string,
  identifier: string,
): string => {
  return `${state.moduleAddress}::${moduleName}::${identifier}`;
};

function getModuleAddressAndName(network: Network): {
  moduleAddress: string;
} {
  switch (network) {
    case Network.DEVNET:
      throw "DEVNET not supported";
    case Network.LOCAL:
      return {
        moduleAddress:
          "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30",
      };
    case Network.TESTNET:
      return {
        moduleAddress:
          "0xecf4655ee96b7280a7fc699d009299fd27d10273c050dbc582aa5844c57fd891",
      };
    case Network.MAINNET:
      // Doesn't work right now.
      return {
        moduleAddress:
          "0x67a614e8df22d397b7a7057d743e6b30f8ef2820c054a391658c06199187fa3c",
      };
    case Network.CUSTOM:
      throw "CUSTOM not supported";
  }
}
