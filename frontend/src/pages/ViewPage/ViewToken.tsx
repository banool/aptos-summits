import { Box, Flex, Link, Text } from "@chakra-ui/react";
import { useEffect, useState } from "react";
import init, { run } from "../../summits/summits";

const CANVAS_ID = "summitcanvas";

export const ViewToken = ({ tokenAddress }: { tokenAddress: string }) => {
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);

  // If this width doesn't match the width used elsewhere, the generation will run
  // differently due to how the values for the mountains are generated (based on
  // width rather than just generating heaps of points and then throwing some away
  // if the actual width is smaller). As such, to scale down the canvas after the
  // fact we do a css transform.
  const renderWidth = 2000;

  // How much of the page the art should take up.
  const artFraction = 0.75;

  const [scale, setScale] = useState(0.1);

  // This hook sets the `scale` that we pass to the CSS transform to make sure the
  // canvas fills 80% of the page.
  useEffect(() => {
    const handleResize = () => {
      const smallerDimension = Math.min(window.innerWidth, window.innerHeight);
      const newScale = (smallerDimension * artFraction) / renderWidth;
      setScale(newScale);
    };

    // Calculate the scale factor on initial render
    handleResize();

    // Add event listener for window resize
    window.addEventListener("resize", handleResize);

    // Cleanup the event listener
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  async function runWasm() {
    setLoading(true);

    // TODO: This is probably how we would we load the wasm from elsewhere.
    // const response = await fetch("http://127.0.0.1:8000/summits_bg.wasm");
    // const wasmArrayBuffer = await response.arrayBuffer();

    await init();
    setLoading(false);
    setLoaded(true);

    // This blocks forever.
    console.log("Generating for token", tokenAddress);
    run(renderWidth, tokenAddress, `#${CANVAS_ID}`);
  }

  let button = null;

  /*
  if (loading) {
    button = <Button isDisabled={true}>Loading...</Button>;
  } else if (!loaded) {
    button = <Button onClick={runWasm}>Load</Button>;
  }
  */

  // Just run the wasm automatically.
  useEffect(() => {
    runWasm();
  }, []);

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
      {loaded && (
        <Box>
          <Box
            textAlign="center"
            paddingLeft={35}
            paddingRight={35}
            paddingBottom={2}
          >
            <Text>Click on the art and then press P to pause / unpause.</Text>
          </Box>
          <Box
            textAlign="center"
            paddingLeft={40}
            paddingRight={40}
            paddingBottom={5}
          >
            <Text>
              Note: Due to some unfortunate portability problems with the RNG
              library + JS + WASM, the art will likely look different here than
              it does for your actual token.{" "}
              <Link
                color={"blueviolet"}
                href="https://github.com/rust-random/rand/issues/1415"
              >
                Learn more
              </Link>
              .
            </Text>
          </Box>
        </Box>
      )}
      {button}
      {!loaded && (
        <Text p={5}>
          Loading. Refresh the page and try again if the image doesn't load in
          ~10 seconds.
        </Text>
      )}
      <Box filter={loading ? "blur(4px)" : "none"}>
        <canvas
          id={CANVAS_ID}
          style={{
            transform: `scale(${scale})`,
            transformOrigin: "top center",
          }}
        ></canvas>
      </Box>
    </Flex>
  );
};
