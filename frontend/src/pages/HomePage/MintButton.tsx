import { Box, Text, Button, Flex, useToast } from "@chakra-ui/react";
import React, { useState } from "react";
import { getModuleIdentifier, useGlobalState } from "../../context/GlobalState";
import {
  TransactionResponseType,
  WriteSetChangeWriteResource,
} from "@aptos-labs/ts-sdk";
import { useWallet } from "@aptos-labs/wallet-adapter-react";
import { Link as ReactRouterLink } from "react-router-dom";
import { Link as ChakraLink } from "@chakra-ui/react";

export const MintButton = () => {
  const { account, signAndSubmitTransaction } = useWallet();
  const [globalState] = useGlobalState();
  const [tokenAddress, setTokenAddress] = useState("");
  const toast = useToast();

  const handleSubmit = async () => {
    const data = {
      function: getModuleIdentifier(
        globalState,
        "summits_token",
        "mint",
      ) as any,
      typeArguments: [],
      functionArguments: [globalState.collectionAddress],
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

      console.log(JSON.stringify(waitResponse));
      console.log("yo");
      console.log(JSON.stringify(waitResponse.changes), null, 2);

      // TODO: A function to get the objects created in a txn would be nice. I don't
      // believe such a function exists still, so I use the event that is emitted for
      // now.
      let tokenAddress = null;
      for (const change of waitResponse.changes) {
        // TODO: Do this type check properly.
        if (change.type !== "write_resource") {
          continue;
        }
        console.log(change);
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

  return (
    <Box p={10}>
      <Flex alignContent="center">
        <Button
          paddingLeft={6}
          paddingRight={6}
          onClick={handleSubmit}
          isDisabled={!buttonEnabled}
        >
          Mint
        </Button>
      </Flex>
      {tokenAddress && (
        <Box>
          <Text>
            {"Token created at "}
            <ChakraLink
              color="lightblue"
              as={ReactRouterLink}
              to={`/${tokenAddress}?network=${globalState.network}`}
            >
              {tokenAddress}
            </ChakraLink>
            {"."}
          </Text>
        </Box>
      )}
    </Box>
  );
};
