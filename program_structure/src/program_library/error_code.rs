use core::fmt;
use std::fmt::Formatter;

use crate::{error_definition::Report, file_definition::{FileLocation, FileID}, ast::Version};

#[derive(Copy, Clone)]
pub enum ReportCode {
    //Parse Errors
    UnclosedComment,
    GenericParsing,
    FileOs,
    NoMain,
    MultipleMain,
    CompilerVersion,
    MissingSemicolon,
    UnrecognizedInclude,
    UnrecognizedVersion,
    UnrecognizedPragma,
    IncludeNotFound,
    IllegalExpression,
    MultiplePragma,
    //
    AssertWrongType,
    ParseFail,
    CompilerVersionError,
    WrongTypesInAssignOperation,
    WrongNumberOfArguments(usize, usize),
    UndefinedFunction,
    UndefinedTemplate,
    UninitializedSymbolInExpression,
    UnableToTypeFunction,
    UnreachableConstraints,
    UnreachableTags,
    UnreachableSignals,
    UnknownIndex,
    UnknownDimension,
    SameFunctionDeclaredTwice,
    SameTemplateDeclaredTwice,
    SameSymbolDeclaredTwice,
    StaticInfoWasOverwritten,
    SignalInLineInitialization,
    SignalOutsideOriginalScope,
    FunctionWrongNumberOfArguments,
    FunctionInconsistentTyping,
    FunctionPathWithoutReturn,
    FunctionReturnError,
    ForbiddenDeclarationInFunction,
    NonHomogeneousArray,
    NonBooleanCondition,
    NonCompatibleBranchTypes,
    NonEqualTypesInExpression,
    NonExistentSymbol,
    NoMainFoundInProject,
    NoCompilerVersionWarning,
    MultipleMainInComponent,
    MainComponentWithTags,
    TemplateCallAsArgument,
    TemplateWrongNumberOfArguments,
    TemplateWithReturnStatement,
    TypeCantBeUseAsCondition,
    EmptyArrayInlineDeclaration,
    PrefixOperatorWithWrongTypes,
    ParallelOperatorWithWrongTypes,
    InfixOperatorWithWrongTypes,
    InvalidArgumentInCall,
    InconsistentReturnTypesInBlock,
    InconsistentStaticInformation,
    InvalidArrayAccess,
    InvalidSignalAccess,
    InvalidTagAccess,
    InvalidTagAccessAfterArray,
    InvalidArraySize,
    InvalidArrayType,
    ForStatementIllConstructed,
    BadArrayAccess,
    AssigningAComponentTwice,
    AssigningASignalTwice,
    NotAllowedOperation,
    ConstraintGeneratorInFunction,
    WrongSignalTags,
    InvalidPartialArray,
    MustBeSingleArithmetic,
    MustBeArithmetic,
    OutputTagCannotBeModifiedOutside,
    MustBeSameDimension,
    ExpectedDimDiffGotDim(usize, usize),
    RuntimeError,
    RuntimeWarning,
    UnknownTemplate,
    NonQuadratic,
    NonConstantArrayLength,
    NonComputableExpression,
    // Constraint analysis codes
    UnconstrainedSignal,
    OneConstraintIntermediate,
    NoOutputInInstance,
    ErrorWat2Wasm,
    CustomGateIntermediateSignalWarning,
    CustomGateConstraintError,
    CustomGateSubComponentError,
    CustomGatesPragmaError,
    CustomGatesVersionError,
    AnonymousCompError,
    TupleError,
    InvalidSignalTagAccess,
}
impl fmt::Display for ReportCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use self::ReportCode::*;
        let string_format = match self {
            ParseFail => "P1000",
            NoMainFoundInProject => "P1001",
            MultipleMainInComponent => "P1002",
            CompilerVersionError => "P1003",
            NoCompilerVersionWarning => "P1004",
            WrongTypesInAssignOperation => "T2000",
            UndefinedFunction => "T2001",
            UndefinedTemplate => "T2002",
            UninitializedSymbolInExpression => "T2003",
            UnableToTypeFunction => "T2004",
            UnreachableConstraints => "T2005",
            SameFunctionDeclaredTwice => "T2006",
            SameTemplateDeclaredTwice => "T2007",
            SameSymbolDeclaredTwice => "T2008",
            StaticInfoWasOverwritten => "T2009",
            SignalInLineInitialization => "T2010",
            SignalOutsideOriginalScope => "T2011",
            FunctionWrongNumberOfArguments => "T2012",
            FunctionInconsistentTyping => "T2013",
            FunctionPathWithoutReturn => "T2014",
            FunctionReturnError => "T2015",
            ForbiddenDeclarationInFunction => "T2016",
            NonHomogeneousArray => "T2017",
            NonBooleanCondition => "T2018",
            NonCompatibleBranchTypes => "T2019",
            NonEqualTypesInExpression => "T2020",
            NonExistentSymbol => "T2021",
            TemplateCallAsArgument => "T2022",
            TemplateWrongNumberOfArguments => "T2023",
            TemplateWithReturnStatement => "T2024",
            TypeCantBeUseAsCondition => "T2025",
            EmptyArrayInlineDeclaration => "T2026",
            PrefixOperatorWithWrongTypes => "T2027",
            ParallelOperatorWithWrongTypes => "T2047",
            InfixOperatorWithWrongTypes => "T2028",
            InvalidArgumentInCall => "T2029",
            InconsistentReturnTypesInBlock => "T2030",
            InconsistentStaticInformation => "T2031",
            InvalidArrayAccess => "T2032",
            InvalidSignalAccess => "T2046",
            InvalidSignalTagAccess => "T2047",
            InvalidTagAccess => "T2048",
            InvalidTagAccessAfterArray => "T2049",
            InvalidArraySize => "T2033",
            InvalidArrayType => "T2034",
            ForStatementIllConstructed => "T2035",
            BadArrayAccess => "T2035",
            AssigningAComponentTwice => "T2036",
            AssigningASignalTwice => "T2037",
            NotAllowedOperation => "T2038",
            ConstraintGeneratorInFunction => "T2039",
            WrongSignalTags => "T2040",
            AssertWrongType => "T2041",
            UnknownIndex => "T2042",
            InvalidPartialArray => "T2043",
            MustBeSingleArithmetic => "T2044",
            ExpectedDimDiffGotDim(..) => "T2045",
            MustBeSameDimension => "T2046",
            MustBeArithmetic => "T2047",
            OutputTagCannotBeModifiedOutside => "T2048",
            UnreachableTags => "T2049",
            UnreachableSignals => "T2050",
            MainComponentWithTags => "T2051",
            RuntimeError => "T3001",
            RuntimeWarning => "T3002",
            UnknownDimension => "T20460",
            UnknownTemplate => "T20461",
            NonQuadratic => "T20462",
            NonConstantArrayLength => "T20463",
            NonComputableExpression => "T20464",
            WrongNumberOfArguments(..) => "T20465",
            // Constraint analysis codes
            UnconstrainedSignal => "CA01",
            OneConstraintIntermediate => "CA02",
            NoOutputInInstance => "CA03",
            ErrorWat2Wasm => "W01",
            CustomGateIntermediateSignalWarning => "CG01",
            CustomGateConstraintError => "CG02",
            CustomGateSubComponentError => "CG03",
            CustomGatesPragmaError => "CG04",
            CustomGatesVersionError => "CG05",
            AnonymousCompError => "TAC01",
            TupleError => "TAC02",
            UnclosedComment => "P01",
            GenericParsing  => "P02",
            FileOs  => "P03",
            NoMain => "P04",
            MultipleMain => "P05",
            CompilerVersion => "P06",
            MissingSemicolon => "P07",
            UnrecognizedInclude => "P08",
            UnrecognizedVersion => "P09",
            UnrecognizedPragma => "P10",
            IllegalExpression => "P11",
            MultiplePragma => "P12",
            IncludeNotFound => "P13",
        };
        f.write_str(string_format)
    }
}




pub fn produce_report_with_message(error_code : ReportCode, msg : String) -> Report {
    match error_code {
        ReportCode::FileOs => {
            Report::error(
            format!("Could not open file {}", msg),
            ReportCode::FileOs,
            )
        }
        ReportCode::IncludeNotFound => {
            Report::error(
                format!(" The file {} to be included has not been found", msg),
                ReportCode::FileOs,
                )
        },
        _ => unreachable!()
    }
}

pub fn produce_generic_report( msg : String, location : FileLocation, file_id : FileID) -> Report {
    let mut report = Report::error(msg, ReportCode::GenericParsing);
    report.add_primary(location, file_id, "Error here".to_string());
    report
}
pub fn produce_compiler_version_report(path : String, required_version : Version, version :  Version) -> Report {
    let report = Report::error(
        format!("File {} requires pragma version {:?} that is not supported by the compiler (version {:?})", path, required_version, version ),
        ReportCode::CompilerVersionError,
    );
    report
}


 pub fn produce_report(error_code: ReportCode, location : FileLocation, file_id : FileID) -> Report {
    use ReportCode::*;
    let report  = match error_code {
            UnclosedComment => {
                let mut report =
                    Report::error("unterminated /* */".to_string(), ReportCode::UnclosedComment);
                report.add_primary(location, file_id, "Comment starts here".to_string());
                report
            }
            NoMain => Report::error(
                "No main specified in the project structure".to_string(),
                ReportCode::NoMainFoundInProject,
            ),
            MultipleMain =>{
                Report::error(
                    "Multiple main components in the project structure".to_string(),
                    ReportCode::MultipleMainInComponent,
                )
            }
            MissingSemicolon => {
                let mut report = Report::error(format!("Missing semicolon"), 
                    ReportCode::MissingSemicolon);
                report.add_primary(location, file_id, "A semicolon is needed here".to_string());
                report
            }
            UnrecognizedInclude => {
                let mut report =
                Report::error("unrecognized argument in include directive".to_string(), ReportCode::UnrecognizedInclude);
            report.add_primary(location, file_id, "this argument".to_string());
            report

            }
            UnrecognizedPragma => {
                let mut report =
                Report::error("unrecognized argument in pragma directive".to_string(), ReportCode::UnrecognizedPragma);
            report.add_primary(location, file_id, "this argument".to_string());
            report

            }        
            UnrecognizedVersion => {
                let mut report =
                Report::error("unrecognized version argument in pragma directive".to_string(), ReportCode::UnrecognizedVersion);
            report.add_primary(location, file_id, "this argument".to_string());
            report
            }      
            IllegalExpression => {
                let mut report =
                Report::error("illegal expression".to_string(), ReportCode::IllegalExpression);
            report.add_primary(location, file_id, "here".to_string());
            report
            }
            MultiplePragma => {
                let mut report =
                Report::error("Multiple pragma directives".to_string(), ReportCode::MultiplePragma);
            report.add_primary(location, file_id, "here".to_string());
            report
            },
            _ => unreachable!(),    
    };
    report
}

