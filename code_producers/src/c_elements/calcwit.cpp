#include <iomanip>
#include <assert.h>
#include "calcwit.hpp"

extern void run(Circom_CalcWit* ctx);

std::string int_to_hex( u64 i )
{
  std::stringstream stream;
  stream << "0x"
         << std::setfill ('0') << std::setw(16)
         << std::hex << i;
  return stream.str();
}

u64 fnv1a(std::string s) {
  u64 hash = 0xCBF29CE484222325LL;
  for(char& c : s) {
    hash ^= u64(c);
    hash *= 0x100000001B3LL;
  }
  return hash;
}

Circom_CalcWit::Circom_CalcWit (Circom_Circuit *aCircuit, uint maxTh) {
  circuit = aCircuit;
  inputSignalAssignedCounter = get_main_input_signal_no();
  inputSignalAssigned = new bool[inputSignalAssignedCounter];
  for (int i = 0; i< inputSignalAssignedCounter; i++) {
    inputSignalAssigned[i] = false;
  }
  signalValues = new FrElement[get_total_signal_no()];
  Fr_str2element(&signalValues[0], "1");
  componentMemory = new Circom_Component[get_number_of_components()];
  circuitConstants = circuit ->circuitConstants;
  templateInsId2IOSignalInfo = circuit -> templateInsId2IOSignalInfo;

  maxThread = maxTh;

  // parallelism
  numThread = 0;

}

Circom_CalcWit::~Circom_CalcWit() {
  // ...
}

void Circom_CalcWit::setInputSignal(u64 h, uint i,  FrElement & val){
  if (inputSignalAssignedCounter == 0) {
    fprintf(stderr, "No more signals to be assigned\n");
    assert(false);
  }
  uint n = get_size_of_input_hashmap();
  uint pos = (uint)(h % (u64)n);
  if (circuit->InputHashMap[pos].hash!=h){
    uint inipos = pos;
    pos++;
    while (pos != inipos) {
      if (circuit->InputHashMap[pos].hash==h) break;
      if (circuit->InputHashMap[pos].hash==0) {
	fprintf(stderr, "Signals not fond\n");
	assert(false);
      }
      pos = (pos+1)%n; 
    }
    if (pos == inipos) {
      fprintf(stderr, "Signals not fond\n");
      assert(false);
    }
  }
  uint si = circuit->InputHashMap[pos].signalid+i;
  if (inputSignalAssigned[si-get_main_input_signal_start()]) {
    fprintf(stderr, "Signal assigned twice: %d\n", si);
    assert(false);
  }
  signalValues[si] = val;
  inputSignalAssigned[si-get_main_input_signal_start()] = true;
  inputSignalAssignedCounter--;
  if (inputSignalAssignedCounter == 0) {
    run(this);
  }
}

