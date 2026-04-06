// Export all command modules
const doctorCommand = require('./doctor');
const initCommand = require('./init');
const validatorStartCommand = require('./validator-start');
const validatorStatusCommand = require('./validator-status');
const validatorInfoCommand = require('./validator-info');
const validatorRegisterCommand = require('./validator-register');
const delegationsCommand = require('./delegations');
const rewardsCommand = require('./rewards');
const walletCommand = require('./wallet');
const balanceCommand = require('./balance');
const transferCommand = require('./transfer');
const txHistoryCommand = require('./tx-history');
const multisigCommand = require('./multisig');
const claimCommand = require('./claim');
const unstakeCommand = require('./unstake');
const stakeCommand = require('./stake');
const stakePositionsCommand = require('./stake-positions');
const networkCommand = require('./network');
const monitorLoop = require('./monitor');
const logsCommand = require('./logs');
const sdkCommand = require('./sdk');
const sdkTestCommand = require('./sdk-test');
const configCommand = require('./config');
const epochCommand = require('./epoch');
const supplyCommand = require('./supply');
const statusCommand = require('./status');
const validatorsListCommand = require('./validators');
const blockhashCommand = require('./blockhash');
const tpsCommand = require('./tps');
const feesCommand = require('./fees');
const apyCommand = require('./apy');
const accountCommand = require('./account');
const priceCommand = require('./price');
const emergencyCommand = require('./emergency');
const snapshotCommand = require('./snapshot');
const nftCommand = require('./nft');

module.exports = {
  doctorCommand,
  initCommand,
  validatorStartCommand,
  validatorStatusCommand,
  validatorInfoCommand,
  validatorRegisterCommand,
  delegationsCommand,
  rewardsCommand,
  walletCommand,
  balanceCommand,
  transferCommand,
  txHistoryCommand,
  multisigCommand,
  claimCommand,
  unstakeCommand,
  stakeCommand,
  stakePositionsCommand,
  networkCommand,
  monitorLoop,
  logsCommand,
  sdkCommand,
  sdkTestCommand,
  configCommand,
  epochCommand,
  supplyCommand,
  statusCommand,
  validatorsListCommand,
  blockhashCommand,
  tpsCommand,
  feesCommand,
  apyCommand,
  accountCommand,
  priceCommand,
  emergencyCommand,
  snapshotCommand,
  nftCommand,
};
