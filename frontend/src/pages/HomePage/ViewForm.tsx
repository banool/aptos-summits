import { Box, Text, Button, Input, Flex } from "@chakra-ui/react";
import React, { useState } from "react";
import { useGlobalState } from "../../context/GlobalState";
import { AccountAddress } from "@aptos-labs/ts-sdk";
import { useNavigate } from "react-router-dom";

export const ViewForm = () => {
  const [globalState] = useGlobalState();
  const [inputValue, setInputValue] = useState("");
  const navigate = useNavigate();

  const handleInputChange = (e: {
    target: { value: React.SetStateAction<string> };
  }) => {
    setInputValue(e.target.value);
  };

  const inputValid = AccountAddress.isValid({
    input: inputValue,
    strict: true,
  }).valid;

  const buttonEnabled = inputValid;

  return (
    <Box>
      <Box paddingBottom={3}>
        <Text>{"View your summit"}</Text>
      </Box>
      <Flex alignContent="center">
        <Input
          placeholder="Enter token address"
          value={inputValue}
          onChange={handleInputChange}
          mb={4}
        />
        <Box ml={4} />
        <Button
          paddingLeft={6}
          paddingRight={6}
          isDisabled={!buttonEnabled}
          onClick={() =>
            navigate(`/${inputValue}?network=${globalState.network}`)
          }
        >
          View
        </Button>
      </Flex>
    </Box>
  );
};
