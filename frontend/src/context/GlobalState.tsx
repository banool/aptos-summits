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
  /** derived from network_value */
  readonly collectionAddress: string;
};

type GlobalActions = {
  selectNetwork: ReturnType<typeof useNetworkSelector>[1];
};

function deriveGlobalState({ network }: { network: Network }): GlobalState {
  const config = new AptosConfig({ network });
  const client = new Aptos(config);
  const { moduleAddress, collectionAddress } = getModuleAddressAndName(network);
  return {
    network,
    client,
    moduleAddress,
    collectionAddress,
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
  collectionAddress: string;
} {
  switch (network) {
    case Network.DEVNET:
      throw "DEVNET not supported";
    case Network.LOCAL:
      return {
        moduleAddress:
          "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30",
        collectionAddress:
          "0x62b2f40e367c990d1d5443515440697daaa6fcf5047c03c2dd479c1c9352aead",
      };
    case Network.TESTNET:
      return {
        moduleAddress:
          "0x0933ba379c760ab471a581f941aa8c01ff34f29a4149277e36b645a24910816a",
        collectionAddress: "todo",
      };
    case Network.MAINNET:
      // Doesn't work right now.
      return {
        moduleAddress:
          "0x81e2e2499407693c81fe65c86405ca70df529438339d9da7a6fc2520142b591e",
        collectionAddress: "todo",
      };
    case Network.CUSTOM:
      throw "CUSTOM not supported";
  }
}
