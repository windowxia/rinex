use crate::cli::Context;

use rtk::prelude::{
    BdModel, Constellation, IonosphereBiasModel, IonosphereBiasModelIter as RTKModelIter, KbModel,
    NgModel,
};

/// Efficient model stream
pub struct IonosphereModelIter<'a> {
    iter: Box<dyn Iterator<Item = IonosphereBiasModel> + 'a>,
}

impl<'a> IonosphereModelIter<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let nav = ctx.data.brdc_navigation().unwrap(); //infaillible

        let header = &nav.header;
        let version = header.version.major;

        match version < 4 {
            true => {
                let t = nav.first_epoch().expect("Invalid empty NAV RINEX");
                Self {
                    iter: {
                        Box::new(header.ionod_correction.iter().filter_map(move |corr| {
                            if let Some(kb) = corr.as_klobuchar() {
                                Some(IonosphereBiasModel::klobuchar(
                                    t,
                                    KbModel {
                                        alpha: kb.alpha,
                                        beta: kb.beta,
                                        h_km: 350.0,
                                    },
                                ))
                            } else if let Some(ng) = corr.as_nequick_g() {
                                Some(IonosphereBiasModel::nequick_g(t, NgModel { a: ng.a }))
                            } else if let Some(bd) = corr.as_bdgim() {
                                Some(IonosphereBiasModel::bdgim(t, BdModel { alpha: bd.alpha }))
                            } else {
                                None
                            }
                        }))
                    },
                }
            },
            false => Self {
                iter: {
                    Box::new(
                        nav.ionod_correction_models()
                            .filter_map(|(t, (_, sv, model))| {
                                if let Some(kb) = model.as_klobuchar() {
                                    Some(IonosphereBiasModel::klobuchar(
                                        *t,
                                        KbModel {
                                            alpha: kb.alpha,
                                            beta: kb.beta,
                                            h_km: {
                                                match sv.constellation {
                                                    Constellation::BeiDou => 375.0,
                                                    _ => 350.0,
                                                }
                                            },
                                        },
                                    ))
                                } else if let Some(ng) = model.as_nequick_g() {
                                    Some(IonosphereBiasModel::nequick_g(*t, NgModel { a: ng.a }))
                                } else if let Some(bd) = model.as_bdgim() {
                                    Some(IonosphereBiasModel::bdgim(
                                        *t,
                                        BdModel { alpha: bd.alpha },
                                    ))
                                } else {
                                    None
                                }
                            }),
                    )
                },
            },
        }
    }
}

impl<'a> RTKModelIter for IonosphereModelIter<'a> {
    fn next(&mut self) -> Option<IonosphereBiasModel> {
        self.iter.next()
    }
}
