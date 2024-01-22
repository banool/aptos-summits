import { Box, Divider, Flex } from "@chakra-ui/react";
import { ViewForm } from "./ViewForm";
import { MintButton } from "./MintButton";

// TODO Align mint button in center, make it bigger.
// TODO Down the line add a way to let people see other peoples' tokens.
// TODO Besides the input button, figure out if they own a token with an indexer query to see if they own a token in the collection
export const HomePage = () => {
  return (
    <Box p={10}>
      <Flex alignContent="center">
        <MintButton />
      </Flex>
      <Divider m={10} />
      <ViewForm />
    </Box>
  );
};
