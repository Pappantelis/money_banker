// User from Google OAuth
export interface User {
  id: string;
  google_id: string;
  email: string;
  f_name: string;
  l_name: string;
  photo_url?: string;
  created_at: string;
  updated_at: string;
}

// Transaction category
export interface Category {
  id: string;
  name: string;
  icon?: string;
  color?: string;
  is_income: boolean;
  user_id: string;
}

// Bank transaction
export interface Transaction {
  id: string;
  user_id: string;
  amount: number;
  store?: string;
  description?: string;
  category_id?: string;
  category?: Category;
  transaction_date: string;
  source: "email" | "manual";
  created_at: string;
}

// For creating new transactions
export interface CreateTransaction {
  amount: number;
  store?: string;
  description?: string;
  category_id?: string;
  transaction_date: string;
  is_income: boolean;
}

// Monthly summary
export interface MonthlySummary {
  income: number;
  expenses: number;
  balance: number;
  transaction_count: number;
}

// Filter options for transactions
export interface TransactionFilter {
  year?: number;
  month?: number;
  category_id?: string;
  source?: "email" | "manual";
}

// App state
export interface AppState {
  user: User | null;
  isLoading: boolean;
  isAuthenticated: boolean;
}
