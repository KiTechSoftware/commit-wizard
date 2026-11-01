# Changelog

## [v0.0.4](https://github.com/KiTechSoftware/commit-wizard/compare/v0.0.3...v0.0.4) - 2026-11-01

### Features

- **branch:** prune merged local branches and merged remote branches ([d338991](https://github.com/KiTechSoftware/commit-wizard/commit/d33899128271bbd8f0fe2bedaabd784210bc679a))
- **branch:** let a ticket or a description alone name the branch ([385d169](https://github.com/KiTechSoftware/commit-wizard/commit/385d169112978122a759c561d8a1414ac395c2a5))
- **branch:** require a description, and a type or ticket only when configured ([e8e435e](https://github.com/KiTechSoftware/commit-wizard/commit/e8e435e9cc745fc95f97e0245ae9ff7c30c4f19e))
- add -T as the short form of --ticket on commit and branch new ([b3d1099](https://github.com/KiTechSoftware/commit-wizard/commit/b3d1099c8312d6d70450b1f7618ba7b58f9bb17e))
- **branch:** name branches from branch.naming.pattern and add --description ([117f857](https://github.com/KiTechSoftware/commit-wizard/commit/117f857cfca73ad9f0540568c516ad5a92497340))
- **branch:** add cw branch to list, switch, create and prune branches ([866127c](https://github.com/KiTechSoftware/commit-wizard/commit/866127c8f8fdee2766728e0bbbf85b52f7e848da))
- **commit:** add ticket references and the -a / -A staging flags ([5899646](https://github.com/KiTechSoftware/commit-wizard/commit/5899646180e84add16919fc0255820183418f110))
- **init:** make --set, --section and --rules do what the manual says ([8c462b0](https://github.com/KiTechSoftware/commit-wizard/commit/8c462b0100c798023bd0a9b495c0c022db193d5a))
- **fix:** add cw fix to correct commit messages that break the rules ([566263b](https://github.com/KiTechSoftware/commit-wizard/commit/566263b1639ec1b2ae1531441517fd869bcc4488))
- **config:** write files atomically and edit TOML surgically ([e73b4f8](https://github.com/KiTechSoftware/commit-wizard/commit/e73b4f8884125aaafde26614015483418b37dbae))
- **config:** add a guided config edit that starts from the current values ([6118bc6](https://github.com/KiTechSoftware/commit-wizard/commit/6118bc6abc867211338e6b728c1f618ac527dc3a))
- **init:** add a guided setup wizard for project and global configs ([97f28b0](https://github.com/KiTechSoftware/commit-wizard/commit/97f28b006b371841cced9e4b4673be212b352389))
- **tag:** guide the tag on a terminal, asking only what the flags left open ([841e056](https://github.com/KiTechSoftware/commit-wizard/commit/841e056b84a829e27aed5e5e529a7fcf2c5ce372))
- **cli:** prompt only on a terminal and make --yes accept defaults ([ec3a9b5](https://github.com/KiTechSoftware/commit-wizard/commit/ec3a9b58a48aaa56de1851ae7f804984bb5fd1da))
- **changelog:** ask what to generate when run on a terminal with no options ([0876827](https://github.com/KiTechSoftware/commit-wizard/commit/0876827964d5d98a81e81f76eb8f806dff9937f0))
- **changelog:** link releases and entries to the repository ([31a2bdc](https://github.com/KiTechSoftware/commit-wizard/commit/31a2bdc0bb5e82e33687be139861b20f2e25c62b))
- **changelog:** honour show_empty_sections/scopes and date annotated tags by tag time ([7291c31](https://github.com/KiTechSoftware/commit-wizard/commit/7291c31561e792a7a66d3e3065577da068c12866))
- **config:** warn about unknown top-level config keys with a suggestion ([741ea8a](https://github.com/KiTechSoftware/commit-wizard/commit/741ea8a6f6cbe4444afbbffbd590616855f416a8))
- **config:** let a project opt in to the global config with [global] inherit ([e590165](https://github.com/KiTechSoftware/commit-wizard/commit/e59016525f0e587bbab071f7eb1b5103212aa7e8))
- **config:** add config list and config list --effective ([89d3a9a](https://github.com/KiTechSoftware/commit-wizard/commit/89d3a9a0654d75f81b05808d1a824d59e03f86fb))
- **config:** support YAML and JSON config files alongside TOML ([7387a62](https://github.com/KiTechSoftware/commit-wizard/commit/7387a627555bf90190098fe997987cdf0dfbb446))
- **changelog:** add the cw changelog command ([4141bcc](https://github.com/KiTechSoftware/commit-wizard/commit/4141bcc8892b612f45011b4319a49a986598792f))
- prepare v0.0.3 (#20) ([e39d031](https://github.com/KiTechSoftware/commit-wizard/commit/e39d031bc8b97e360456a4951458528e4be2507f))
- prepare release of v0.0.2 (#16) ([10cdc28](https://github.com/KiTechSoftware/commit-wizard/commit/10cdc28237749d360723a8a19f7754a4e92bd2d6))

### Bug Fixes

- **config:** read Go date layouts token by token and accept [rules] in rules files ([0b8e4ef](https://github.com/KiTechSoftware/commit-wizard/commit/0b8e4ef0b75281691f967bb728f29a73cac5c311))
- **cli:** honour tag --branch and refuse scopes when they are disabled ([fc9687b](https://github.com/KiTechSoftware/commit-wizard/commit/fc9687bcf5e225ea825e40cf54c7d742c3d550b3))
- **cli:** accept --global after the config subcommand and hyphenated tag suffixes ([c737378](https://github.com/KiTechSoftware/commit-wizard/commit/c737378a534ccf9b7ab2a6b2d5400347fd839957))
- **config:** accept lowercase bump levels ([ade00e1](https://github.com/KiTechSoftware/commit-wizard/commit/ade00e1828d0d13efd72057040b7fa878901fafc))
- **check:** exit non-zero when any commit breaks the rules ([80001b5](https://github.com/KiTechSoftware/commit-wizard/commit/80001b5a09a885c30298218a0ef0787466393467))
- **init:** make init config and init rules create real files ([e40fc90](https://github.com/KiTechSoftware/commit-wizard/commit/e40fc90c6ba043050beed2901b7caff3eb72af89))
- **lint:** drop a needless Some/? in the changelog link helper ([985c157](https://github.com/KiTechSoftware/commit-wizard/commit/985c1578f446acf7de1570e9c674a7a57d26efbf))
- **config:** apply the env layer and keep config -g on the global file ([1969c99](https://github.com/KiTechSoftware/commit-wizard/commit/1969c99ee7bcd7bce8ee56f265e6a3fb52d29730))
- **version:** use prerelease labels that end in a number as written ([df93e60](https://github.com/KiTechSoftware/commit-wizard/commit/df93e60bd2ccf301ab48701839fa93d933d0549c))
- **deps:** drop the unused scriba figlet feature to remove the vulnerable zip ([c5b2060](https://github.com/KiTechSoftware/commit-wizard/commit/c5b2060dba1739dca039126c05fea03e8450de44))

### Documentation

- **branch:** document prune --merged and --remote ([f6bef92](https://github.com/KiTechSoftware/commit-wizard/commit/f6bef922e776a4c2ea6a8caa0fc2980f8ef9f0e4))
- **branch:** describe the description-or-ticket rule and the guided prompts ([818bfa5](https://github.com/KiTechSoftware/commit-wizard/commit/818bfa50e905c7164b3e884f360530b89221e333))
- describe which branch flags are required and the two new settings ([29b60d4](https://github.com/KiTechSoftware/commit-wizard/commit/29b60d43c46e0ba1b509030bd78a0a7da91e63fe))
- describe the branch naming template and the --description flag ([3e32262](https://github.com/KiTechSoftware/commit-wizard/commit/3e32262a2722ec3fd970d76f57ae931558303edc))
- add the cw branch page and list it in the command tables ([87b38c0](https://github.com/KiTechSoftware/commit-wizard/commit/87b38c08548601eb63037ccf6079003d1500b9ed))
- document tickets and the staging flags, and drop their reserved notices ([7f828bb](https://github.com/KiTechSoftware/commit-wizard/commit/7f828bb27d6461040bc5885439352cf69c1fbe03))
- correct registry cache, rules precedence and date format notes ([b3d4e75](https://github.com/KiTechSoftware/commit-wizard/commit/b3d4e7520e651521e46d33d1fb6d48457d850099))
- show the real cw bump output in the getting-started guide ([1b30f91](https://github.com/KiTechSoftware/commit-wizard/commit/1b30f91ff17a5c6c9e505415cefef713d55c3900))
- document settable config keys, init --set and registry flags ([fc44f48](https://github.com/KiTechSoftware/commit-wizard/commit/fc44f4820d72921bf51165d0060a7267420a93aa))
- align commit, push, doctor and add pages with behaviour ([a8f5ef0](https://github.com/KiTechSoftware/commit-wizard/commit/a8f5ef0cd7d7a3e9b94ea422f9d4fc097b581613))
- correct footer_keys, header description and reserved pr defaults ([06ec012](https://github.com/KiTechSoftware/commit-wizard/commit/06ec0120d6ba07a33489fb937e62792b3cac4bf4))
- correct ticket, branch and pr defaults and explain release candidates ([420c505](https://github.com/KiTechSoftware/commit-wizard/commit/420c505900142ea0f53519f4b435444b16431d05))
- fix config examples that did not load and add an environment reference ([5f3b432](https://github.com/KiTechSoftware/commit-wizard/commit/5f3b432450aabcf514fb995b8610bcbb58a00332))
- mirror the README on docs.rs, add ARCHITECTURE.md and module docs ([191c10f](https://github.com/KiTechSoftware/commit-wizard/commit/191c10f12163153b54ba3b683f0857feaca9fb70))
- hide the internal modules from the API docs and require docs on what remains ([3eabee8](https://github.com/KiTechSoftware/commit-wizard/commit/3eabee8788e747325f664fbaeb68fa50e81ecf11))
- prepare the crate and its documentation for a crates.io release ([508363b](https://github.com/KiTechSoftware/commit-wizard/commit/508363bfc50d6f331ec69b22ea1723c0e62cfc6b))

### Tests

- build file URLs and path assertions that work on Windows ([955047e](https://github.com/KiTechSoftware/commit-wizard/commit/955047e277a2493474b59b93c733ea372cdf069a))
- isolate the Windows profile folders as well as HOME ([2378142](https://github.com/KiTechSoftware/commit-wizard/commit/2378142bb700c053da08232704977bdbebcbc3a2))
- keep the shared test helpers available on Windows ([723ea22](https://github.com/KiTechSoftware/commit-wizard/commit/723ea22c56914c5aa374228eb5d57d7f0e33f686))

### CI

- make the Windows job required now that its suite passes ([8b6a1bd](https://github.com/KiTechSoftware/commit-wizard/commit/8b6a1bd2bc95812167be61f9f0863d57eff3b32a))
- build and test on Windows, and report where cw keeps its files ([8db76a9](https://github.com/KiTechSoftware/commit-wizard/commit/8db76a911ca2c1f82e33d8c96c5e973dabd641f6))
- update Dependabot configuration for pull requests ([7af3b55](https://github.com/KiTechSoftware/commit-wizard/commit/7af3b5569ddc7216c5cd6ad393b2a4bf906c9561))
- add docker and actions scanning (#9) ([a076d7d](https://github.com/KiTechSoftware/commit-wizard/commit/a076d7d1ca47c523600602e746c29dddfbaf845a))

### Chores

- remove unused env constants and orphaned source files ([7c90b78](https://github.com/KiTechSoftware/commit-wizard/commit/7c90b780caa8a1b4ba3c86fb2c8fa0a3c749a452))
- **ci:** fix the rust cache path and add dev-git-sync.sh ([4101e20](https://github.com/KiTechSoftware/commit-wizard/commit/4101e2096edb36b3f4174bc97cda523eed0c4998))
- sync main into develop ([582c03c](https://github.com/KiTechSoftware/commit-wizard/commit/582c03c70230ab4247e46250cbfbb355f54d79ad))
- sync main into develop ([f3c0c9b](https://github.com/KiTechSoftware/commit-wizard/commit/f3c0c9b933f9594b108df8999e78b0135330b5e7))
- sync/v0.0.2 (#19) ([505cdc6](https://github.com/KiTechSoftware/commit-wizard/commit/505cdc6dd9aa1476479e8071151284c378dc9df7))
- add v0.1.0 code published to cargo (#10) ([7fc4f51](https://github.com/KiTechSoftware/commit-wizard/commit/7fc4f51832c1f173f3bd1e7d2d526701684dbd06))

