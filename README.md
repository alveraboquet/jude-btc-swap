# JUDE to BTC Atomic Swap

This repository hosts an MVP for atomically swapping BTC to JUDE.
It implements the protocol described in section 3 of [this](https://arxiv.org/abs/2101.12332) paper.

Currently, swaps are only offered in one direction with the `swap` CLI on the buying side (send BTC, receive JUDE).
We are working on implementing a protocol where JUDE moves first, but are currently blocked by advances on judecoin itself.
You can read [this blogpost](https://www.judecoin.io/blog) for more information.

## Quick Start

1. Download the [latest `swap` binary release](https://github.com/judecoin/jude-btc-swap.git) for your operating system.
2. Find a seller to swap with:

```shell
./swap --testnet list-sellers
```

3. Swap with a seller:

```shell
./swap --testnet buy-jude --receive-address <YOUR JUDECOIN ADDRESS> --change-address <YOUR BITCOIN CHANGE ADDRESS> --seller <SELLER MULTIADDRESS>
```

For more detailed documentation on the CLI, see [this README](./docs/cli/README.md).

## Becoming a Market Maker

Swapping of course needs two parties - and the CLI is only one of them: The taker that occasionally starts a swap with a market maker.

If you are interested in becoming a market maker you will want to run the second binary provided in this repository: `asb` - the Automated Swap Backend.
Detailed documentation for the `asb` can be found [in this README](./docs/asb/README.md).

## Safety

This software is using cryptography that has not been formally audited.
While we do our best to make it safe, it is up to the user to evaluate whether or not it is safe to use for their purposes.
Please also see section 15 and 16 of the [license](./LICENSE).

Keep in mind that swaps are complex protocols, it is recommended to _not_ do anything fancy when moving coins in and out.
It is not recommended to bump fees when swapping because it can have unpredictable side effects.

## Contributing

We are encourage community contributions whether it be a bug fix or an improvement to the documentation.
Please have a look at the [contribution guidelines](./CONTRIBUTING.md).

## Contact

Feel free to reach out to us in the [COMIT-judecoin Matrix channel](https://www.judecoin.io/).
