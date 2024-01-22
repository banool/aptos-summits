import { DefaultABITable, ExtractStructType } from "@thalalabs/surf";
import { COLLECTION_ABI, TOKEN_ABI, GENR_ABI } from "./abis";

type ABITAble = DefaultABITable & {
  "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::summits_collection": typeof COLLECTION_ABI;
  "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::summits_token": typeof TOKEN_ABI;
  "0x296102a3893d43e11de2aa142fbb126377120d7d71c246a2f95d5b4f3ba16b30::genr": typeof GENR_ABI;

};

// export type Piece = ExtractStructType<ABITAble, typeof CHESS_ABI, "Piece">;
