import { Box, Text, Button, Flex, useToast } from "@chakra-ui/react";
import React, { useState } from "react";
import { getModuleIdentifier, useGlobalState } from "../../context/GlobalState";
import {
  TransactionResponseType,
  WriteSetChangeWriteResource,
} from "@aptos-labs/ts-sdk";
import { useWallet } from "@aptos-labs/wallet-adapter-react";
import { Link as ReactRouterLink, useNavigate } from "react-router-dom";
import { Link as ChakraLink } from "@chakra-ui/react";
import "../../css/buttons.css";
import { useTokens } from "../../api/useTokens";

export const MintButton = () => {
  const { account, signAndSubmitTransaction } = useWallet();
  const [globalState] = useGlobalState();
  const [tokenAddress, setTokenAddress] = useState("");
  const toast = useToast();
  const navigate = useNavigate();

  // Add query to look up if the user has a token already.
  // isLoading is only true the first time, not on refresh.
  // https://redux-toolkit.js.org/rtk-query/usage/queries#frequently-used-query-hook-return-values
  // We assume the user only has a single token, even if this isn't enforced on chain
  // right now.
  const { data, isLoading, isError } = useTokens(account?.address as any, {
    enabled: account !== null,
  });

  const handleSubmit = async () => {
    if (account === null) {
      toast({
        title: "Connect your wallet",
        status: "warning",
        duration: 4000,
        isClosable: true,
      });
      return;
    }

    const data = {
      function: getModuleIdentifier(
        globalState,
        "summits_token",
        "mint",
      ) as any,
      typeArguments: [],
      functionArguments: [],
    };

    try {
      let submissionResponse = await signAndSubmitTransaction({
        sender: account!.address,
        data,
      });
      const waitResponse = await globalState.client.waitForTransaction({
        transactionHash: submissionResponse.hash,
        options: { checkSuccess: true, waitForIndexer: true },
      });

      // Needed to make the type checker happy.
      if (waitResponse.type !== TransactionResponseType.User) {
        throw new Error("Transaction was unexpectedly the wrong type");
      }

      // TODO: A function to get the objects created in a txn would be nice. I don't
      // believe such a function exists still, so I use the event that is emitted for
      // now.
      let tokenAddress = null;
      for (const change of waitResponse.changes) {
        // TODO: Do this type check properly.
        if (change.type !== "write_resource") {
          continue;
        }
        let c: WriteSetChangeWriteResource = change as any;
        if (c.data.type == "0x4::token::Token") {
          tokenAddress = c.address;
        }
      }

      if (tokenAddress === null) {
        throw new Error("Couldn't find token address from mint function");
      }

      setTokenAddress(tokenAddress);
    } catch (error) {
      console.log(`Error minting token: ${JSON.stringify(error)}`);
      toast({
        title: "Error minting token",
        status: "error",
        duration: 4000,
        isClosable: true,
      });
    }
  };

  // TODO: Check that the user is allowlisted.
  const buttonEnabled = account !== null;

  let buttonText;
  let onClick = null;
  if (!buttonEnabled) {
    buttonText = "Connect Wallet";
    onClick = () => navigate(`/${tokenAddress}?network=${globalState.network}`);
  } else if (isLoading) {
    buttonText = "Loading...";
  } else if (tokenAddress) {
    buttonText = "Reveal";
    onClick = () => navigate(`/${tokenAddress}?network=${globalState.network}`);
  } else if (data && data.length > 0) {
    buttonText = "View";
    onClick = () => navigate(`/${data[0]}?network=${globalState.network}`);
  } else {
    buttonText = "Mint";
    onClick = handleSubmit;
  }

  return (
    <Box p={10}>
      <Flex alignContent="center">
        <Box
          width="300px"
          height="250px"
          className="mountain-button"
          onClick={onClick !== null ? onClick : undefined}
        >
          <span>{buttonText}</span>
        </Box>
      </Flex>
    </Box>
  );
};
