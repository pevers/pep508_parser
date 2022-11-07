//! # pep508_parser
//! 
//! PEP-508 parser by creating a Parsing Expression Grammar (PEG) for PEP-508 strings
//! See: https://peps.python.org/pep-0508/

use pest::Parser;
use pest_derive::Parser;
use semver::VersionReq;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error(transparent)]
    InvalidInput(#[from] pest::error::Error<Rule>),

    #[error("invalid version spec")]
    InvalidVersionSpec(#[from] semver::Error),
}
#[derive(Debug, PartialEq)]
pub struct Dependency {
    /// Name of the package
    pub name: String,
    /// Version limits
    pub version: VersionReq,
    /// Optional extras that expand the dependencies of the distribution to enable optional features
    pub extras: Option<Vec<String>>,
    /// Optional URI specification
    pub uri: Option<String>,
    /// Optional markers
    pub markers: Option<Vec<String>>,
}

#[derive(Parser)]
#[grammar = "pep508.pest"]
struct Pep508Parser;

/// Parse a PEP-508 string and return a Dependency
pub fn parse(input: &str) -> Result<Dependency, ParserError> {
    let main = Pep508Parser::parse(Rule::main, input)?
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();
    match main.as_rule() {
        Rule::name_req | Rule::url_req => {
            let (mut name, mut extras_list, mut versionspec, mut uri, mut markers) =
                ("", vec![], Some("*"), None, vec![]);
            main.into_inner().flatten().for_each(|p| match p.as_rule() {
                Rule::name => name = p.as_span().as_str(),
                Rule::versionspec => versionspec = Some(p.as_span().as_str()),
                Rule::extras_list => {
                    extras_list = p
                        .as_span()
                        .as_str()
                        .replace(" ", "")
                        .split(",")
                        .map(str::to_string)
                        .collect()
                }
                Rule::marker_expr => markers.push(p.as_span().as_str().to_string()),
                Rule::URI_reference => uri = Some(p.as_span().as_str().to_string()),
                _ => {}
            });
            Ok(Dependency {
                name: name.to_string(),
                version: VersionReq::parse(versionspec.unwrap())?,
                extras: Some(extras_list.clone()),
                markers: Some(markers.clone()),
                uri: uri.clone(),
            })
        }
        _ => unreachable!("grammar is inconsistent with the parser"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("A")]
    #[case("A.B-C_D")]
    #[case("aa")]
    #[case("name")]
    #[case("name<=1")]
    #[case("name>=3")]
    #[case("name>=3,<2")]
    #[case("name@http://foo.com")]
    #[case("name [fred,bar] @ http://foo.com ; python_version=='2.7'")]
    #[case("name[quux, strange];python_version<'2.7' and platform_version=='2'")]
    #[case("name; os_name=='a' or os_name=='b'")]
    #[case("name; os_name=='a' and os_name=='b' or os_name=='c'")]
    #[case("name; os_name=='a' and (os_name=='b' or os_name=='c')")]
    #[case("name; os_name=='a' or os_name=='b' and os_name=='c'")]
    #[case("name; (os_name=='a' or os_name=='b') and os_name=='c'")]
    #[case("name[quux, strange];python_version<'2.7' and platform_version=='2'")]
    fn test_main(#[case] input: &str) {
        Pep508Parser::parse(Rule::main, input).unwrap();
    }

    #[test]
    fn test_parse_extras() {
        let parsed =
            parse("name[quux, strange];python_version<'2.7' and platform_version=='2'").unwrap();
        assert_eq!(
            parsed,
            Dependency {
                name: String::from("name"),
                version: VersionReq::parse("*").unwrap(),
                extras: Some(vec![String::from("quux"), String::from("strange")]),
                uri: None,
                markers: Some(vec![
                    String::from("python_version<'2.7'"),
                    String::from("platform_version=='2'")
                ])
            }
        );
    }

    #[test]
    fn test_uri_spec() {
        let parsed = parse("name [fred,bar] @ http://foo.com ; python_version=='2.7'").unwrap();
        assert_eq!(
            parsed,
            Dependency {
                name: String::from("name"),
                version: VersionReq::parse("*").unwrap(),
                extras: Some(vec![String::from("fred"), String::from("bar")]),
                uri: Some("http://foo.com".to_string()),
                markers: Some(vec![String::from("python_version=='2.7'")])
            }
        );
    }
}
