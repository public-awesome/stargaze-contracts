/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.16.5.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Addr, Uint128, ConfigResponse, MinterConfigForEmpty, Empty, Coin, ExecuteMsg, Timestamp, Uint64, Decimal, InstantiateMsg, CreateMinterMsgForNullable_Empty, CollectionParams, CollectionInfoForRoyaltyInfoResponse, RoyaltyInfoResponse, MinterParamsForNullable_Empty, QueryMsg, StatusResponse, Status } from "./BaseMinter.types";
export interface BaseMinterReadOnlyInterface {
  contractAddress: string;
  config: () => Promise<ConfigResponse>;
  status: () => Promise<StatusResponse>;
}
export class BaseMinterQueryClient implements BaseMinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.config = this.config.bind(this);
    this.status = this.status.bind(this);
  }

  config = async (): Promise<ConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
  status = async (): Promise<StatusResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      status: {}
    });
  };
}
export interface BaseMinterInterface extends BaseMinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  mint: ({
    tokenUri
  }: {
    tokenUri: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateTradingStartTime: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class BaseMinterClient extends BaseMinterQueryClient implements BaseMinterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.mint = this.mint.bind(this);
    this.updateTradingStartTime = this.updateTradingStartTime.bind(this);
  }

  mint = async ({
    tokenUri
  }: {
    tokenUri: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint: {
        token_uri: tokenUri
      }
    }, fee, memo, funds);
  };
  updateTradingStartTime = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_trading_start_time: {}
    }, fee, memo, funds);
  };
}