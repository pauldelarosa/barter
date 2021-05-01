use crate::execution::fill::{FillEvent, Fees};
use crate::execution::error::ExecutionError;
use chrono::Utc;
use crate::portfolio::order::OrderEvent;

/// Generates a result [FillEvent] by executing an [OrderEvent].
pub trait FillGenerator {
    /// Return a [FillEvent] from executing the input [OrderEvent].
    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError>;
}

/// Configuration for constructing a [SimulatedExecution] via the new() constructor method.
pub struct Config {
    /// Simulated fee percentage to be used for each [Fees] field in decimal form (eg/ 0.01 for 1%)
    pub simulated_fees_pct: Fees,
}

/// Simulated execution handler that executes [OrderEvent]s to generate [FillEvent]s via a
/// simulated broker interaction.
pub struct SimulatedExecution {
    simulated_fees_pct: Fees,
}

impl FillGenerator for SimulatedExecution {
    fn generate_fill(&self, order: &OrderEvent) -> Result<FillEvent, ExecutionError> {
        // Assume (for now) that all orders are filled at the market price
        let fill_value_gross = SimulatedExecution::calculate_fill_value_gross(order);

        Ok(FillEvent {
            event_type: FillEvent::EVENT_TYPE,
            trace_id: order.trace_id,
            timestamp: Utc::now(),
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            decision: order.decision.clone(),
            quantity: order.quantity,
            fill_value_gross,
            fees: self.calculate_fees(&fill_value_gross),
        })
    }
}

impl SimulatedExecution {
    /// Constructs a new [SimulatedExecution] component.
    pub fn new(cfg: &Config) -> Self {
        Self {
            simulated_fees_pct: cfg.simulated_fees_pct,
        }
    }

    /// Calculates the simulated gross fill value (excluding TotalFees) based on the input [OrderEvent].
    fn calculate_fill_value_gross(order: &OrderEvent) -> f64 {
        order.quantity.abs() * order.close
    }

    /// Calculates the simulated [Fees] a [FillEvent] will incur, based on the input [OrderEvent].
    fn calculate_fees(&self, fill_value_gross: &f64) -> Fees {
        Fees {
            exchange: self.simulated_fees_pct.exchange * fill_value_gross,
            slippage: self.simulated_fees_pct.slippage * fill_value_gross,
            network: self.simulated_fees_pct.network * fill_value_gross
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_ok_fill_event_with_valid_order_event_provided() {
        let simulated_execution = SimulatedExecution::new();

        let mut input_order = OrderEvent::default();
        input_order.quantity = 10.0;
        input_order.close = 10.0;

        let actual_result = simulated_execution.generate_fill(&input_order);

        assert!(actual_result.is_ok() && actual_result.unwrap().fill_value_gross == 100.0)
    }

    #[test]
    fn should_calculate_fill_value_gross_correctly() {
        let mut input_order = OrderEvent::default();
        input_order.quantity = 100.0;
        input_order.close = 10.0;

        let actual = SimulatedExecution::calculate_fill_value_gross(&input_order);

        let expected = 100.0 * 10.0;

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_calculate_fill_value_gross_correctly_with_negative_order_quantity_provided() {
        let mut input_order = OrderEvent::default();
        input_order.quantity = -(100.0);
        input_order.close = 10.0;

        let actual = SimulatedExecution::calculate_fill_value_gross(&input_order);

        let expected = (100.0 * 10.0) as f64;

        assert_eq!(actual, expected)
    }
}