import type { ShopDepositAccount } from './shop-deposit-account';

export interface ShopDepositAccountResponse {
  code: string;
  message: string;
  /** Server-owned request correlation id. */
  requestId: string;
  data: ShopDepositAccount;
}
