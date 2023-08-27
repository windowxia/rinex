use crate::plot::{build_chart_epoch_axis, generate_markers, PlotContext};
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::quality::QcContext;
use rinex::{observation::*, prelude::*};
use std::collections::HashMap;

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
    let mut phase_plot = false;
    let mut doppler_plot = false;
    let mut ssi_plot = false;
    let mut pr_plot = false;
    /*
     * One plot per physics
     */
    for observable in primary_data.observable() {
        match observable {
            Observable::Phase(_) => {
                if !phase_plot {
                    plot_context.add_cartesian2d_plot("Phase cycles", "Carrier whole cycles");
                    phase_plot = true;
                }
            },
            Observable::Doppler(_) => {
                if !doppler_plot {
                    plot_context.add_cartesian2d_plot("Doppler shifts", "Doppler");
                    doppler_plot = true;
                }
            },
            Observable::SSI(_) => {
                if !doppler_plot {
                    plot_context.add_cartesian2d_plot("Signal Strength", "SSI [dB]");
                    ssi_plot = true;
                }
            },
            Observable::PseudoRange(_) => {
                if !doppler_plot {
                    plot_context.add_cartesian2d_plot("Pseudo Range", "Pseudo Range");
                    pr_plot = true;
                }
            },
            _ => unreachable!(),
        }
        // Design plot markers : one per signal
        //let total = total_markers.get(&observable).unwrap_or(&(1 as usize));
        let markers = generate_markers(20);
        /*
         * SSI observation special case
         *  in case NAV augmentation was provided
         *  it is useful to visualize elevation
         *  angles at the same time. In this case,
         *  we use a dual Y axis plot.
         */
        if observable.is_ssi_observable() && has_nav {
            /*
             * Design one color per Sv
             */
            for (sv_index, sv) in primary_data.sv().enumerate() {
                let (data_x, data_y): (Vec<Epoch>, Vec<f64>) = match observable {
                    Observable::SSI(_) => (
                        primary_data
                            .ssi()
                            .filter_map(|((e, _), svnn, obs, _value)| {
                                if svnn == sv && obs == observable {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        primary_data
                            .ssi()
                            .filter_map(|((_e, _), svnn, obs, value)| {
                                if svnn == sv && obs == observable {
                                    Some(value)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    _ => unreachable!(),
                };
            }
        } else {
            /*
             * Design one color per Sv
             */
            for (sv_index, sv) in primary_data.sv().enumerate() {
                let (data_x, data_y): (Vec<Epoch>, Vec<f64>) = match observable {
                    Observable::Phase(_) => (
                        primary_data
                            .carrier_phase()
                            .filter_map(|((e, _), svnn, obs, _value)| {
                                if svnn == sv && obs == observable {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        primary_data
                            .carrier_phase()
                            .filter_map(|((_e, _), svnn, obs, value)| {
                                if svnn == sv && obs == observable {
                                    Some(value)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    Observable::PseudoRange(_) => (
                        primary_data
                            .pseudo_range()
                            .filter_map(|((e, _), svnn, obs, _value)| {
                                if svnn == sv && obs == observable {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        primary_data
                            .pseudo_range()
                            .filter_map(|((_e, _), svnn, obs, value)| {
                                if svnn == sv && obs == observable {
                                    Some(value)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    Observable::SSI(_) => (
                        primary_data
                            .ssi()
                            .filter_map(|((e, _), svnn, obs, _value)| {
                                if svnn == sv && obs == observable {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        primary_data
                            .ssi()
                            .filter_map(|((_e, _), svnn, obs, value)| {
                                if svnn == sv && obs == observable {
                                    Some(value)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    Observable::Doppler(_) => (
                        primary_data
                            .doppler()
                            .filter_map(|((e, _), svnn, obs, _value)| {
                                if svnn == sv && obs == observable {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        primary_data
                            .doppler()
                            .filter_map(|((_e, _), svnn, obs, value)| {
                                if svnn == sv && obs == observable {
                                    Some(value)
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    _ => unreachable!(),
                };
                let trace = build_chart_epoch_axis(
                    &format!("{}({})", sv, observable),
                    Mode::Markers,
                    data_x,
                    data_y,
                )
                .marker(Marker::new().symbol(markers[sv_index & 10].clone()))
                .visible({
                    if sv_index > 4 {
                        Visible::LegendOnly
                    } else {
                        Visible::True
                    }
                });
            }
        }
    }
}
