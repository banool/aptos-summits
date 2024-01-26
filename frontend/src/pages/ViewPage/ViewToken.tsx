import { Box, Button, Flex, Text } from "@chakra-ui/react";
import { useState } from "react";
import init, { run } from "../../summits/summits";

const CANVAS_ID = "summitcanvas";

export const ViewToken = ({ tokenAddress }: { tokenAddress: string }) => {
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);

  async function runWasm() {
    setLoading(true);
    // TODO: This is probably how we would we load the wasm from elsewhere.
    // const response = await fetch("http://127.0.0.1:8000/summits_bg.wasm");
    // const wasmArrayBuffer = await response.arrayBuffer();
    const width = 1600;
    await init();
    setLoading(false);
    setLoaded(true);
    // This blocks forever.
    run(width, tokenAddress, `#${CANVAS_ID}`);
  }

  let button = null;
  if (loading) {
    button = <Button isDisabled={true}>Loading...</Button>;
  } else if (!loaded) {
    button = <Button onClick={runWasm}>Load</Button>;
  }

  // TODO: Find a way to make the canvas only a certain size.
  return (
    <Flex
      w="100%"
      flex="1"
      justifyContent="center"
      alignItems="center"
      flexDirection="column"
    >
      {button}
      {!loaded && (
        <Text p={5}>
          Refresh the page and try again if the image doesn't load in ~10 seconds.
        </Text>
      )}
      <Box w={400} h={400} filter={loading ? "blur(4px)" : "none"}>
        <canvas id={CANVAS_ID}></canvas>
      </Box>
    </Flex>
  );
};
