import type { MonthlySummary } from "../types";

interface DashboardProps {
  summary: MonthlySummary | null;
  selectedDate: { year: number; month: number };
  onDateChange: (year: number, month: number) => void;
  onAddClick: () => void;
}

const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December"
];

export function Dashboard({ summary, selectedDate, onDateChange, onAddClick }: DashboardProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat("el-GR", {
      style: "currency",
      currency: "EUR",
    }).format(amount);
  };

  const handlePrevMonth = () => {
    if (selectedDate.month === 1) {
      onDateChange(selectedDate.year - 1, 12);
    } else {
      onDateChange(selectedDate.year, selectedDate.month - 1);
    }
  };

  const handleNextMonth = () => {
    if (selectedDate.month === 12) {
      onDateChange(selectedDate.year + 1, 1);
    } else {
      onDateChange(selectedDate.year, selectedDate.month + 1);
    }
  };

  return (
    <div className="dashboard">
      <div className="summary-cards">
        <div className="summary-card income">
          <span className="card-label">Income</span>
          <span className="card-value">
            {formatCurrency(summary?.income ?? 0)}
          </span>
        </div>

        <div className="summary-card expenses">
          <span className="card-label">Expenses</span>
          <span className="card-value">
            {formatCurrency(summary?.expenses ?? 0)}
          </span>
        </div>

        <div className="summary-card balance">
          <span className="card-label">Balance</span>
          <span className={`card-value ${(summary?.balance ?? 0) >= 0 ? "positive" : "negative"}`}>
            {(summary?.balance ?? 0) >= 0 ? "+" : ""}
            {formatCurrency(summary?.balance ?? 0)}
          </span>
        </div>
      </div>

      <div className="date-navigation">
        <button className="nav-button" onClick={handlePrevMonth}>
          ←
        </button>
        <span className="current-date">
          {MONTHS[selectedDate.month - 1]} {selectedDate.year}
        </span>
        <button className="nav-button" onClick={handleNextMonth}>
          →
        </button>

        <button className="add-button" onClick={onAddClick}>
          + Add Transaction
        </button>
      </div>
    </div>
  );
}
