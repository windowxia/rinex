use crate::plot::{build_chart_epoch_axis, generate_markers, PlotContext};
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::quality::QcContext;
use rinex::{observation::*, prelude::*};
use std::collections::HashMap;

fn observable_to_physics(observable: &Observable) -> String {
    if observable.is_phase_observable() {
        "Phase".to_string()
    } else if observable.is_doppler_observable() {
        "Doppler".to_string()
    } else if observable.is_ssi_observable() {
        "Signal Strength".to_string()
    } else {
        "Pseudo Range".to_string()
    }
}

/*
 * Plots given Observation RINEX content
 */
pub fn plot_observation(ctx: &QcContext, plot_context: &mut PlotContext) {
    let has_nav = ctx.has_navigation_data();
    let primary_data = ctx.primary_data();
    /*
     * Plot receiver clock data, if such data exists
     */
    if primary_data.receiver_clock().count() > 0 {
        plot_context.add_cartesian2d_plot("Rcvr Clock", "Clock bias [s]");
        let data_x_ok: Vec<_> = primary_data
            .receiver_clock()
            .filter_map(|((e, flag), _)| if flag.is_ok() { Some(e) } else { None })
            .collect();
        let data_y_ok: Vec<_> = primary_data
            .receiver_clock()
            .filter_map(
                |((_, flag), bias)| {
                    if flag.is_ok() {
                        Some(bias)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis("clk", Mode::Markers, data_x_ok, data_y_ok)
            .marker(Marker::new().symbol(MarkerSymbol::TriangleUp));
    }
    /*
     * We'll design one marker per signal,
     * we need to determine total of signals per physics
     */
    let mut total_markers: HashMap<Observable, usize> = HashMap::new();
    for observable in primary_data.observable() {
        if let Some(count) = total_markers.get_mut(&observable) {
            *count += 1;
        } else {
            total_markers.insert(observable.clone(), 1);
        }
    }
    /*
     * One plot per physics
     */
    for observable in primary_data.observable() {
        let (y_label, iter) = match observable {
            Observable::Phase(_) => ("Carrier cycles", primary_data.phase()),
            Observable::Doppler(_) => ("Doppler Shifts", primary_data.doppler()),
            Observable::SSI(_) => ("Power [dB]", primary_data.ssi()),
            Observable::PseudoRange(_) => ("Pseudo Range", primary_data.pseudo_range()),
            _ => unreachable!(),
        };
        // Design plot markers : one per signal
        let total = total_markers.get(&observable).unwrap_or(&(1 as usize));
        let markers = generate_markers(*total);
        /*
         * SSI observation special case
         *  in case NAV augmentation was provided
         *  it is useful to visualize elevation
         *  angles at the same time. In this case,
         *  we use a dual Y axis plot.
         */
        if observable.is_ssi_observable() && has_nav {
            plot_context.add_cartesian2d_2y_plot(&observable.to_string(), y_label, "Elevation [°]");
        } else {
            plot_context.add_cartesian2d_plot(&observable.to_string(), y_label);
            /*
             * Design one color per Sv PRN#
             */
            for (sv_index, sv) in primary_data.sv().enumerate() {
                let data_x: Vec<Epoch> = primary_data
                    .observation()
                    .flat_map(|((e, _), (_, vehicles))| {
                        vehicles.iter().filter_map(|(svnn, observables)| {
                            if *svnn == sv {
                                Some(observables)
                            } else {
                                None
                            }
                        })
                    })
                    .filter_map(|(obs, obsdata)| {
                        if obs == observable {
                            Some(obsdata)
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        }
    }
}
