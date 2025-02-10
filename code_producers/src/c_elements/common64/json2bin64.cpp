#include <iostream>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <sys/stat.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>
#include <nlohmann/json.hpp>
#include <vector>

typedef uint64_t u64;
typedef uint32_t u32;
typedef uint8_t u8;

using json = nlohmann::json;

//only for the main inputs
struct __attribute__((__packed__)) HashSignalInfo {
    u64 hash;
    u64 signalid; 
    u64 signalsize; 
};

u64 fnv1a(std::string s) {
  u64 hash = 0xCBF29CE484222325LL;
  for(char& c : s) {
    hash ^= u64(c);
    hash *= 0x100000001B3LL;
  }
  return hash;
}

bool check_valid_number(std::string & s, uint base){
  bool is_valid = true;
  if (base == 16){
    for (uint i = 0; i < s.size(); i++){
      is_valid &= (
        ('0' <= s[i] && s[i] <= '9') || 
        ('a' <= s[i] && s[i] <= 'f') ||
        ('A' <= s[i] && s[i] <= 'F')
      );
    }
  } else{
    for (uint i = 0; i < s.size(); i++){
      is_valid &= ('0' <= s[i] && s[i] < char(int('0') + base));
    }
  }
  return is_valid;
}

void json2FrElements (json val, std::vector<u64> & vval){
  if (!val.is_array()) {
    u64 v;
    std::string s_aux, s;
    uint base;
    if (val.is_string()) {
      s_aux = val.get<std::string>();
      std::string possible_prefix = s_aux.substr(0, 2);
      if (possible_prefix == "0b" || possible_prefix == "0B"){
        s = s_aux.substr(2, s_aux.size() - 2);
        base = 2; 
      } else if (possible_prefix == "0o" || possible_prefix == "0O"){
        s = s_aux.substr(2, s_aux.size() - 2);
        base = 8; 
      } else if (possible_prefix == "0x" || possible_prefix == "0X"){
        s = s_aux.substr(2, s_aux.size() - 2);
        base = 16;
      } else{
        s = s_aux;
        base = 10;
      }
      if (!check_valid_number(s, base)){
        std::ostringstream errStrStream;
        errStrStream << "Invalid number in JSON input: " << s_aux << "\n";
	      throw std::runtime_error(errStrStream.str() );
      }
    } else if (val.is_number()) {
        double vd = val.get<double>();
        std::stringstream stream;
        stream << std::fixed << std::setprecision(0) << vd;
        s = stream.str();
        base = 10;
    } else {
        std::ostringstream errStrStream;
        errStrStream << "Invalid JSON type\n";
	      throw std::runtime_error(errStrStream.str() );
    }
    vval.push_back(strtoull(s.c_str(), NULL, base));
  } else {
    for (uint i = 0; i < val.size(); i++) {
      json2FrElements (val[i], vval);
    }
  }
}

HashSignalInfo* loadMap(std::string const &datFileName) {
  HashSignalInfo *InputHashMap = new HashSignalInfo[256]; //parametrized

    int fd;
    struct stat sb;

    fd = open(datFileName.c_str(), O_RDONLY);
    if (fd == -1) {
        std::cout << ".dat file not found: " << datFileName << "\n";
        throw std::system_error(errno, std::generic_category(), "open");
    }
    
    if (fstat(fd, &sb) == -1) {          /* To obtain file size */
        throw std::system_error(errno, std::generic_category(), "fstat");
    }

    u8* bdata = (u8*)mmap(NULL, sb.st_size, PROT_READ , MAP_PRIVATE, fd, 0);
    close(fd);

    InputHashMap = new HashSignalInfo[256]; //parametrized
    uint dsize = 256*sizeof(HashSignalInfo); //parametrized
    memcpy((void *)(InputHashMap), (void *)bdata, dsize);

    munmap(bdata, sb.st_size);
    
    return InputHashMap;
}

uint getInputSignalHashPosition(HashSignalInfo *InputHashMap, u64 h) {
  uint n = 256; //parametrized
  uint pos = (uint)(h % (u64)n);
  if (InputHashMap[pos].hash!=h){
    uint inipos = pos;
    pos = (pos+1)%n; 
    while (pos != inipos) {
      if (InputHashMap[pos].hash == h) return pos;
      if (InputHashMap[pos].signalid == 0) {
	fprintf(stderr, "Signal not found\n");
	assert(false);
      }
      pos = (pos+1)%n; 
    }
    fprintf(stderr, "Signal not found\n");
    assert(false);
  }
  return pos;
}

u64 getInputSignalSize(HashSignalInfo *InputHashMap, u64 h) {
  uint pos = getInputSignalHashPosition(InputHashMap,h);
  return InputHashMap[pos].signalsize;
}

json::value_t check_type(std::string prefix, json in){
  if (not in.is_array()) {
      return in.type();
    } else {
    if (in.size() == 0) return json::value_t::null;
    json::value_t t = check_type(prefix, in[0]);
    for (uint i = 1; i < in.size(); i++) {
      if (t != check_type(prefix, in[i])) {
	fprintf(stderr, "Types are not the same in the the key %s\n",prefix.c_str());
	assert(false);
      }
    }
    return t;
  }
}

void qualify_input(std::string prefix, json &in, json &in1);

void qualify_input_list(std::string prefix, json &in, json &in1){
    if (in.is_array()) {
      for (uint i = 0; i<in.size(); i++) {
	  std::string new_prefix = prefix + "[" + std::to_string(i) + "]";
	  qualify_input_list(new_prefix,in[i],in1);
	}
    } else {
	qualify_input(prefix,in,in1);
    }
}

void qualify_input(std::string prefix, json &in, json &in1) {
  if (in.is_array()) {
    if (in.size() > 0) {
      json::value_t t = check_type(prefix,in);
      if (t == json::value_t::object) {
	qualify_input_list(prefix,in,in1);
      } else {
	in1[prefix] = in;
      }
    } else {
      in1[prefix] = in;
    }
  } else if (in.is_object()) {
    for (json::iterator it = in.begin(); it != in.end(); ++it) {
      std::string new_prefix = prefix.length() == 0 ? it.key() : prefix + "." + it.key();
      qualify_input(new_prefix,it.value(),in1);
    }
  } else {
    in1[prefix] = in;
  }
}

u64* loadJson(HashSignalInfo *input_hash, std::string filename) {
  std::ifstream inStream(filename);
  json jin;
  inStream >> jin;
  json j;

  u64* input_list = new u64[68602]; // parametrized
  //std::cout << jin << std::endl;
  std::string prefix = "";
  qualify_input(prefix, jin, j);
  //std::cout << j << std::endl;
  uint InputStart = 38; //parametrized
  u64 nItems = j.size();
  // printf("Items : %llu\n",nItems);
  for (json::iterator it = j.begin(); it != j.end(); ++it) {
    //std::cout << it.key() << " => " << it.value() << '\n';
    u64 h = fnv1a(it.key());
    std::vector<u64> v;
    json2FrElements(it.value(),v);
    uint pos = getInputSignalHashPosition(input_hash,h);
    uint signalSize = input_hash[pos].signalsize;
    uint input_position = input_hash[pos].signalid - InputStart;
    //std::cout << pos << ":"  << signalSize << ":"  << input_position << std::endl;
    if (v.size() < signalSize) {
	std::ostringstream errStrStream;
	errStrStream << "Error loading signal " << it.key() << ": Not enough values\n";
	throw std::runtime_error(errStrStream.str() );
    }
    if (v.size() > signalSize) {
	std::ostringstream errStrStream;
	errStrStream << "Error loading signal " << it.key() << ": Too many values\n";
	throw std::runtime_error(errStrStream.str() );
    }
    for (uint i = 0; i<v.size(); i++){
      input_list[input_position+i] = v[i];
    }
  }
  return input_list;
}

void writeBinInput(u64 *input_list, std::string datFileName) {
    FILE *write_ptr;

    write_ptr = fopen(datFileName.c_str(),"wb");
    uint input_size = 68602; //parametrized
    fwrite(input_list, 8, input_size, write_ptr); 
    fclose(write_ptr);
}

int main (int argc, char *argv[]) {
  std::string cl(argv[0]);
  if (argc!=4) {
    std::cout << "Usage: " << "json2bin <circuit.dat> <input.json> <input.dat>\n";
  } else {
    std::string datfile(argv[1]);
    std::string jsonfile(argv[2]);
    std::string binfile(argv[3]);
  
    // auto t_start = std::chrono::high_resolution_clock::now();

     HashSignalInfo *input_hash = loadMap(datfile);
     /* for (uint i = 0; i <256; i++) {
       std::cout << i << ": " << input_hash[i].hash << ": " << input_hash[i].signalid << ": " << input_hash[i].signalsize  << std::endl;
       } */
     u64* input_list = loadJson(input_hash, jsonfile);
     //std::cout << "after load" << std::endl;
     writeBinInput(input_list,binfile);
  }
}
