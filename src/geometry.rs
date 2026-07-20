use crate::appearance::{Material, Texture};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub thetype: GeometryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lod: Option<String>,
    pub boundaries: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<HashMap<String, Material>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<HashMap<String, Texture>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<usize>,
    #[serde(rename = "transformationMatrix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation_matrix: Option<Value>,
}
impl Geometry {
    pub(crate) fn update_geometry_boundaries(&mut self, violdnew: &mut HashMap<usize, usize>) {
        match self.thetype {
            GeometryType::MultiPoint => {
                let a: Vec<usize> = serde_json::from_value(self.boundaries.clone()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    let kk = violdnew.get(&x);
                    if kk.is_none() {
                        let l = violdnew.len();
                        violdnew.insert(*x, l);
                        a2[i] = l;
                    } else {
                        let kk = kk.unwrap();
                        a2[i] = *kk;
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiLineString => {
                let a: Vec<Vec<usize>> = serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        // r.push(z);
                        let kk = violdnew.get(&y);
                        if kk.is_none() {
                            let l = violdnew.len();
                            violdnew.insert(*y, l);
                            a2[i][j] = l;
                        } else {
                            let kk = kk.unwrap();
                            a2[i][j] = *kk;
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let a: Vec<Vec<Vec<usize>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            let kk = violdnew.get(&z);
                            if kk.is_none() {
                                let l = violdnew.len();
                                violdnew.insert(*z, l);
                                a2[i][j][k] = l;
                            } else {
                                let kk = kk.unwrap();
                                a2[i][j][k] = *kk;
                            }
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::Solid => {
                let a: Vec<Vec<Vec<Vec<usize>>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            for (l, zz) in z.iter().enumerate() {
                                let kk = violdnew.get(&zz);
                                if kk.is_none() {
                                    let l2 = violdnew.len();
                                    violdnew.insert(*zz, l2);
                                    a2[i][j][k][l] = l2;
                                } else {
                                    let kk = kk.unwrap();
                                    a2[i][j][k][l] = *kk;
                                }
                            }
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let a: Vec<Vec<Vec<Vec<Vec<usize>>>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            for (l, zz) in z.iter().enumerate() {
                                for (m, zzz) in zz.iter().enumerate() {
                                    let kk = violdnew.get(&zzz);
                                    if kk.is_none() {
                                        let l2 = violdnew.len();
                                        violdnew.insert(*zzz, l2);
                                        a2[i][j][k][l][m] = l2;
                                    } else {
                                        let kk = kk.unwrap();
                                        a2[i][j][k][l][m] = *kk;
                                    }
                                }
                            }
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::GeometryInstance => {
                let a: Vec<usize> = serde_json::from_value(self.boundaries.clone()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    let kk = violdnew.get(&x);
                    if kk.is_none() {
                        let l = violdnew.len();
                        violdnew.insert(*x, l);
                        a2[i] = l;
                    } else {
                        let kk = kk.unwrap();
                        a2[i] = *kk;
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
        }
    }

    pub(crate) fn offset_geometry_boundaries(&mut self, offset: usize) {
        match self.thetype {
            GeometryType::MultiPoint => {
                let a: Vec<usize> = serde_json::from_value(self.boundaries.clone()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    a2[i] = *x + offset;
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiLineString => {
                let a: Vec<Vec<usize>> = serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        // r.push(z);
                        a2[i][j] = *y + offset;
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let a: Vec<Vec<Vec<usize>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            a2[i][j][k] = *z + offset;
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::Solid => {
                let a: Vec<Vec<Vec<Vec<usize>>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            for (l, zz) in z.iter().enumerate() {
                                a2[i][j][k][l] = *zz + offset;
                            }
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let a: Vec<Vec<Vec<Vec<Vec<usize>>>>> =
                    serde_json::from_value(self.boundaries.take()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    for (j, y) in x.iter().enumerate() {
                        for (k, z) in y.iter().enumerate() {
                            for (l, zz) in z.iter().enumerate() {
                                for (m, zzz) in zz.iter().enumerate() {
                                    a2[i][j][k][l][m] = *zzz + offset;
                                }
                            }
                        }
                    }
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
            GeometryType::GeometryInstance => {
                let a: Vec<usize> = serde_json::from_value(self.boundaries.clone()).unwrap();
                let mut a2 = a.clone();
                for (i, x) in a.iter().enumerate() {
                    a2[i] = *x + offset;
                }
                self.boundaries = serde_json::to_value(&a2).unwrap();
            }
        }
    }

    pub(crate) fn update_material(&mut self, m_oldnew: &mut HashMap<usize, usize>) {
        match &mut self.material {
            Some(x) => {
                for (_key, mat) in &mut *x {
                    //-- material.value
                    if mat.value.is_some() {
                        let thevalue: usize = mat.value.unwrap();
                        let r = m_oldnew.get(&thevalue);
                        if r.is_none() {
                            let l = m_oldnew.len();
                            m_oldnew.insert(thevalue, l);
                            mat.value = Some(l);
                        } else {
                            let r2 = r.unwrap();
                            mat.value = Some(*r2);
                        }
                        continue;
                    }
                    //-- else it's material.values (which differs per geom type)
                    match self.thetype {
                        GeometryType::MultiPoint | GeometryType::MultiLineString => (),
                        GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                            if mat.values.is_some() {
                                let a: Vec<Option<usize>> =
                                    serde_json::from_value(mat.values.take().into()).unwrap();
                                let mut a2 = a.clone();
                                for (i, x) in a.iter().enumerate() {
                                    if x.is_some() {
                                        let y2 = m_oldnew.get(&x.unwrap());
                                        if y2.is_none() {
                                            let l = m_oldnew.len();
                                            m_oldnew.insert(x.unwrap(), l);
                                            a2[i] = Some(l);
                                        } else {
                                            let y2 = y2.unwrap();
                                            a2[i] = Some(*y2);
                                        }
                                    }
                                }
                                mat.values = Some(serde_json::to_value(&a2).unwrap());
                            }
                        }
                        GeometryType::Solid => {
                            if mat.values.is_some() {
                                let a: Vec<Vec<Option<usize>>> =
                                    serde_json::from_value(mat.values.take().into()).unwrap();
                                let mut a2 = a.clone();
                                for (i, x) in a.iter().enumerate() {
                                    for (j, y) in x.iter().enumerate() {
                                        if y.is_some() {
                                            let y2 = m_oldnew.get(&y.unwrap());
                                            if y2.is_none() {
                                                let l = m_oldnew.len();
                                                m_oldnew.insert(y.unwrap(), l);
                                                a2[i][j] = Some(l);
                                            } else {
                                                let y2 = y2.unwrap();
                                                a2[i][j] = Some(*y2);
                                            }
                                        }
                                    }
                                }
                                mat.values = Some(serde_json::to_value(&a2).unwrap());
                            }
                        }
                        GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                            if mat.values.is_some() {
                                let a: Vec<Vec<Vec<Option<usize>>>> =
                                    serde_json::from_value(mat.values.take().into()).unwrap();
                                let mut a2 = a.clone();
                                for (i, x) in a.iter().enumerate() {
                                    for (j, y) in x.iter().enumerate() {
                                        for (k, z) in y.iter().enumerate() {
                                            if z.is_some() {
                                                let y2 = m_oldnew.get(&z.unwrap());
                                                if y2.is_none() {
                                                    let l = m_oldnew.len();
                                                    m_oldnew.insert(z.unwrap(), l);
                                                    a2[i][j][k] = Some(l);
                                                } else {
                                                    let y2 = y2.unwrap();
                                                    a2[i][j][k] = Some(*y2);
                                                }
                                            }
                                        }
                                    }
                                }
                                mat.values = Some(serde_json::to_value(&a2).unwrap());
                            }
                        }
                        GeometryType::GeometryInstance => todo!(),
                    }
                }
                self.material = Some(x.clone());
            }
            None => (),
        }
    }
    pub(crate) fn update_texture(
        &mut self,
        t_oldnew: &mut HashMap<usize, usize>,
        t_v_oldnew: &mut HashMap<usize, usize>,
        offset: usize,
    ) {
        match &mut self.texture {
            Some(x) => {
                for (_key, tex) in &mut *x {
                    match self.thetype {
                        GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                            let a: Vec<Vec<Vec<Option<usize>>>> =
                                serde_json::from_value(tex.values.take().into()).unwrap();
                            let mut a2 = a.clone();
                            for (i, x) in a.iter().enumerate() {
                                for (j, y) in x.iter().enumerate() {
                                    for (k, z) in y.iter().enumerate() {
                                        if z.is_some() {
                                            let thevalue: usize = z.unwrap();
                                            if k == 0 {
                                                let y2 = t_oldnew.get(&thevalue);
                                                if y2.is_none() {
                                                    let l = t_oldnew.len();
                                                    t_oldnew.insert(thevalue, l);
                                                    a2[i][j][k] = Some(l);
                                                } else {
                                                    let y2 = y2.unwrap();
                                                    a2[i][j][k] = Some(*y2);
                                                }
                                            } else {
                                                let y2 = t_v_oldnew.get(&thevalue);
                                                if y2.is_none() {
                                                    let l = t_v_oldnew.len();
                                                    t_v_oldnew.insert(thevalue, l + offset);
                                                    a2[i][j][k] = Some(l);
                                                } else {
                                                    let y2 = y2.unwrap();
                                                    a2[i][j][k] = Some(*y2);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            tex.values = Some(serde_json::to_value(&a2).unwrap());
                        }
                        GeometryType::Solid => {
                            let a: Vec<Vec<Vec<Vec<Option<usize>>>>> =
                                serde_json::from_value(tex.values.take().into()).unwrap();
                            let mut a2 = a.clone();
                            for (i, x) in a.iter().enumerate() {
                                for (j, y) in x.iter().enumerate() {
                                    for (k, z) in y.iter().enumerate() {
                                        for (l, zz) in z.iter().enumerate() {
                                            if zz.is_some() {
                                                let thevalue: usize = zz.unwrap();
                                                if l == 0 {
                                                    let y2 = t_oldnew.get(&thevalue);
                                                    if y2.is_none() {
                                                        let l2 = t_oldnew.len();
                                                        t_oldnew.insert(thevalue, l2);
                                                        a2[i][j][k][l] = Some(l2);
                                                    } else {
                                                        let y2 = y2.unwrap();
                                                        a2[i][j][k][l] = Some(*y2);
                                                    }
                                                } else {
                                                    let y2 = t_v_oldnew.get(&thevalue);
                                                    if y2.is_none() {
                                                        let l2 = t_v_oldnew.len();
                                                        t_v_oldnew.insert(thevalue, l2 + offset);
                                                        a2[i][j][k][l] = Some(l2);
                                                    } else {
                                                        let y2 = y2.unwrap();
                                                        a2[i][j][k][l] = Some(*y2);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            tex.values = Some(serde_json::to_value(&a2).unwrap());
                        }
                        _ => todo!(),
                    }
                }
            }
            None => (),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Vertex {
    x: i64,
    y: i64,
    z: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeometryTemplates {
    pub templates: Vec<Geometry>,
    #[serde(rename = "vertices-templates")]
    pub vertices_templates: Value,
}
