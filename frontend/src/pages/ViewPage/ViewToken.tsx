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

    await init();
    setLoading(false);
    setLoaded(true);

    // This doesn't work. Even if you do change the size of the canvas via the browser,
    // it doesn't display correctly. There has to be a way to let a canvas render to
    // the size it wants but constrain it with the parent.
    /*
    setInterval(() => {
      const canvas = document.getElementById(CANVAS_ID) as HTMLCanvasElement;
      if (canvas) {
        canvas.width = width;
        canvas.height = width;
      }
    }, 100);
    */

    // If this width doesn't match the width used elsewhere, the generation will run
    // differently due to how the values for the mountains are generated (based on
    // width rather than just generating heaps of points and then throwing some away
    // if the actual width is smaller).
    const width = 700;

    // This blocks forever.
    run(width, tokenAddress, `#${CANVAS_ID}`);
  }

  let button = null;
  if (loading) {
    button = <Button isDisabled={true}>Loading...</Button>;
  } else if (!loaded) {
    button = <Button onClick={runWasm}>Load</Button>;
  }

  // The w and h for the box wrapping the canvas don't constrain the size of the
  // canvas, it just creates a sort of "window" into the canvas.
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
          Refresh the page and try again if the image doesn't load in ~10
          seconds.
        </Text>
      )}
      <Box filter={loading ? "blur(4px)" : "none"}>
        <canvas id={CANVAS_ID}></canvas>
      </Box>
    </Flex>
  );
};
