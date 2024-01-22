import { ViewToken } from "./ViewToken";
import { useParams } from "react-router-dom";

export const GamePage = () => {
  // Get the object address from the URL using react-router-dom.
  const { token_address } = useParams();

  if (token_address === undefined) {
    return (
      <div>Token address is undefined, you arrived at this page by accident!</div>
    );
  }

  return <ViewToken tokenAddress={token_address} />;
};
