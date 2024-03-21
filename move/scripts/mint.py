"""
First, download Ecosystem Summit January 2024 - Feedback + wallet addresses as a csv file.

Run the script from move/ with --dry-run first:

python scripts/mint.py --profile local ~/Downloads/data.csv --dry-run

If the addresses look good, run it for real:

python scripts/mint.py --profile local ~/Downloads/data.csv
"""

import argparse
import csv
import json
import logging
import subprocess
import random
import time
import urllib.request

import yaml

logging.basicConfig(level="INFO", format="%(asctime)s - %(levelname)s - %(message)s")


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("path")
    parser.add_argument("--profile", required=True)
    parser.add_argument("--assume-yes", action="store_true")
    parser.add_argument("--randomize", action="store_true", help="Randomize the order")
    parser.add_argument("-d", "--debug", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    return args


def main():
    args = parse_args()

    if args.debug:
        logging.setLevel("DEBUG")

    with open(".aptos/config.yaml") as f:
        config = yaml.safe_load(f)

    contract_address = "0x" + config["profiles"][args.profile]["account"]

    raw = []
    with open(args.path) as csvfile:
        reader = csv.reader(csvfile)
        for row in reader:
            address = row[6]
            if not address:
                continue
            if address == "not found":
                continue
            raw.append(address)

    addresses = []
    for address in raw:
        address = address.lower()
        if address.startswith("0x"):
            address = address[2:]
        # Confirm the address is valid hex.
        try:
            int(address, 16)
        except:
            logging.warning(
                f"Not an address, trying to look it up as a name: {address}"
            )
            maybe_address = name_to_address(address)
            try:
                int(maybe_address, 16)
                logging.info(f"Name {address} is 0x{maybe_address}")
                address = maybe_address
            except:
                logging.warning(f"Invalid address, not an ANS name either: {address}")
                continue
        address = address.zfill(64)
        address = "0x" + address
        addresses.append(address)

    logging.info(f"Minting to {len(addresses)} addresses")

    # Remove duplicates while preserving order.
    addresses = list(dict.fromkeys(addresses))

    if args.randomize:
        random.shuffle(addresses)

    function_id = f"{contract_address}::summits_token::mint_to"
    for address in addresses:
        logging.info(f"Minting to {address}")
        if args.dry_run:
            continue
        try:
            cmd = [
                "aptos",
                "move",
                "run",
                "--profile",
                args.profile,
                "--function-id",
                function_id,
                "--args",
                f"address:{address}",
            ]
            if args.assume_yes:
                cmd.append("--assume-yes")
                # Give the indexer some time, just in case.
                # time.sleep(3)
            subprocess.run(
                cmd,
                check=True,
            )
            logging.info(f"Minted to {address}")
        except subprocess.CalledProcessError:
            # We expect minting to fail if the address already has a token so we just
            # warn if something goes wrong.
            logging.warning(f"Minting to {address} failed")


def name_to_address(name):
    # Constructing the URL
    url = (
        f"https://www.aptosnames.com/api/mainnet/v1/address/{urllib.parse.quote(name)}"
    )

    # Making the request
    req = urllib.request.Request(url)

    # Handling the response
    with urllib.request.urlopen(req) as response:
        if response.status == 200:
            # Reading and decoding the response
            response_body = response.read()
            data = json.loads(response_body.decode("utf-8"))

            # Remove leading 0x since we add it back later.
            addr = data.get("address")
            if addr:
                return addr[2:]
            return None
        else:
            return None


if __name__ == "__main__":
    main()
