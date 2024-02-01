"""
First, download Ecosystem Summit January 2024 - Feedback + wallet addresses as a csv file.

Run the script from move/ with --dry-run first:

python scripts/mint.py --profile local ~/Downloads/data.csv --dry-run

If the addresses look good, run it for real:

python scripts/mint.py --profile local ~/Downloads/data.csv
"""

import argparse
import csv
import logging
import subprocess
import yaml


logging.basicConfig(level="INFO", format="%(asctime)s - %(levelname)s - %(message)s")


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("path")
    parser.add_argument("--profile", required=True)
    parser.add_argument("--assume-yes", action="store_true")
    parser.add_argument("-d", "--debug", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    return args


def main():
    args = parse_args()

    if args.debug:
        logging.setLevel("DEBUG")

    addresses = []
    with open(args.path) as csvfile:
        reader = csv.reader(csvfile)
        for row in reader:
            address = row[6]
            addresses.append(address)

    logging.info(f"Minting to {len(addresses)} addresses")

    addresses = [address[2:] for address in addresses if address.startswith("0x")]
    addresses = [address.lower() for address in addresses]
    addresses = [address.zfill(64) for address in addresses]
    addresses = [f"0x{address}" for address in addresses]

    with open(".aptos/config.yaml") as f:
        config = yaml.safe_load(f)

    contract_address = "0x" + config["profiles"][args.profile]["account"]

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
            subprocess.run(
                cmd,
                check=True,
            )
            logging.info(f"Minted to {address}")
        except subprocess.CalledProcessError:
            # We expect minting to fail if the address already has a token so we just
            # warn if something goes wrong.
            logging.warning(f"Minting to {address} failed")


if __name__ == "__main__":
    main()
