use super::DAG;
use constraint_writers::debug_writer::DebugWriter;

struct SlotStatistics {
    number_of_constraints: usize,
    number_of_intermediates: usize,
    number_of_inputs: usize,
    number_of_outputs: usize,
    number_of_components: usize,
    times_reached: usize
}
impl Default for SlotStatistics {
    fn default() -> Self {
        SlotStatistics {
            number_of_constraints: 0,
            number_of_intermediates: 0,
            number_of_inputs: 0,
            number_of_outputs: 0,
            number_of_components: 0,
            times_reached: 0
        }
    }
}

pub fn write_statistics(dag: &DAG,debug: &DebugWriter) -> Result<(),()>{
    use std::io::Write;
    let mut writer = debug.build_statistics_file()?;
    let mut node_statistics = Vec::with_capacity(dag.number_of_nodes());
    for _i in 0..dag.number_of_nodes() {
        node_statistics.push(SlotStatistics::default());
    }
    for i in 0..dag.number_of_nodes() {
        node_statistics[i].number_of_outputs = dag.nodes[i].number_of_outputs();
        node_statistics[i].number_of_inputs = dag.nodes[i].number_of_inputs();
        node_statistics[i].number_of_intermediates = dag.nodes[i].number_of_intermediates();
        node_statistics[i].number_of_constraints = dag.nodes[i].constraints().len();
        node_statistics[i].number_of_components = dag.adjacency[i].len();
        for arrow in &dag.adjacency[i] {
            let pointer = arrow.goes_to;
            node_statistics[pointer].times_reached += 1;
        }
    }
    for (index,node_statistic) in node_statistics.iter().enumerate(){
        let splitter = "***************************************\n".to_string();
        let title = format!("NODE NUMBER: {}\n",index);
        let number_of_constraints = format!("Number of constraints: {}\n",node_statistic.number_of_constraints);
        let number_of_intermediates = format!("Number of intermediates: {}\n", node_statistic.number_of_intermediates);
        let number_of_inputs = format!("Number of inputs: {}\n",node_statistic.number_of_inputs);
        let number_of_outputs = format!("Number of outputs: {}\n",node_statistic.number_of_outputs);
        let number_of_components = format!("Number of components: {}\n",node_statistic.number_of_components);
        let times_reached = format!("Times reached: {}\n",node_statistic.times_reached);

        writer.write_all(splitter.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(title.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(number_of_constraints.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(number_of_intermediates.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(number_of_inputs.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(number_of_outputs.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(number_of_components.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
        writer.write_all(times_reached.as_bytes()).map_err(|_err| {})?;
        writer.flush().map_err(|_err| {})?;
    }
    Result::Ok(())
}