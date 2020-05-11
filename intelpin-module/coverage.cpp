#include <stdio.h>
#include <map>
#include <string>
#include <signal.h>
#include <stdlib.h>
#include <fstream>
#include "pin.H"
using std::string;

std::map<ADDRINT, std::map<ADDRINT, unsigned long> > cflows;
KNOB<string> KnobOutputFile(KNOB_MODE_WRITEONCE, "pintool", "o", "interceptsegv.out", "output file");
std::ofstream Out;

static BOOL SigFunc(THREADID, INT32, CONTEXT *, BOOL, const EXCEPTION_INFO *, void *);

static BOOL SigFunc(THREADID tid, INT32 sig, CONTEXT *ctxt, BOOL hasHandler,
    const EXCEPTION_INFO *exception, void *)
{

  ADDRINT crash_pc = PIN_GetContextReg(ctxt, REG_INST_PTR);

    if (sig == SIGSEGV) {
      IMG test_img = IMG_FindByAddress(crash_pc);
      ADDRINT img_addr_pc = IMG_LowAddress(test_img);

      ADDRINT crash_base_pc = crash_pc-img_addr_pc;
      Out << crash_base_pc << std::endl;
    }

    return TRUE;
}

static BOOL callbackSignals(THREADID tid, INT32 sig, CONTEXT *ctxt, BOOL hasHandler,
    const EXCEPTION_INFO *exception, void *)
{
      printf("[EXCEPTION] ");
      printf("Got other Signal, at PC 0x%08llx\n", PIN_GetContextReg(ctxt, REG_INST_PTR));
    
    return TRUE;   
}

unsigned long cflow_count   = 0;

static void count_cflow(ADDRINT ip, ADDRINT target)
{
  PIN_LockClient();
  IMG img_target = IMG_FindByAddress(target);
  IMG img_ip = IMG_FindByAddress(ip);
  PIN_UnlockClient();
  ADDRINT img_addr_target = IMG_LowAddress(img_target);
  ADDRINT img_addr_ip = IMG_LowAddress(img_ip);

  // Subtract offsets to avoid ASLR
  cflows[target-img_addr_target][ip-img_addr_ip]++;
  cflow_count++;
}

static void instrument_insn(INS ins, void *v)
{
  if(!INS_IsControlFlow(ins)) return; // important! to check whether this is a control-flow instruction

  IMG img = IMG_FindByAddress(INS_Address(ins));

  if(!IMG_Valid(img) || !IMG_IsMainExecutable(img)) return;

  INS_InsertPredicatedCall(
    ins, IPOINT_TAKEN_BRANCH, (AFUNPTR)count_cflow, 
    IARG_INST_PTR, IARG_BRANCH_TARGET_ADDR,
    IARG_END
  );

  if(INS_HasFallThrough(ins)) {
    INS_InsertPredicatedCall(
      ins, IPOINT_AFTER, (AFUNPTR)count_cflow, 
      IARG_INST_PTR, IARG_FALLTHROUGH_ADDR, 
      IARG_END
    );
  }
}

static void print_results(INT32 code, void *v)
{
  ADDRINT ip, target;
  unsigned long count;
  std::map<ADDRINT, std::map<ADDRINT, unsigned long> >::iterator i;
  std::map<ADDRINT, unsigned long>::iterator j;

  for(i = cflows.begin(); i != cflows.end(); i++) {
    target = i->first;
    for(j = i->second.begin(); j != i->second.end(); j++) {
      ip = j->first;
      count = j->second;
      Out << target << " <- " << ip << std::endl;

    } 
  }
}


int main(int argc, char *argv[])
{
  PIN_InitSymbols();
  if(PIN_Init(argc,argv)) {
    return 1;
  }
  
  Out.open(KnobOutputFile.Value().c_str());

  INS_AddInstrumentFunction(instrument_insn, NULL);
  PIN_AddFiniFunction(print_results, NULL);

  PIN_InterceptSignal(SIGSEGV, SigFunc, 0);
  PIN_InterceptSignal(SIGHUP,  callbackSignals, 0);
  PIN_InterceptSignal(SIGINT,  callbackSignals, 0);
  PIN_InterceptSignal(SIGQUIT, callbackSignals, 0);
  PIN_InterceptSignal(SIGILL,  callbackSignals, 0);
  PIN_InterceptSignal(SIGABRT, callbackSignals, 0);
  PIN_InterceptSignal(SIGFPE,  callbackSignals, 0);
  PIN_InterceptSignal(SIGKILL, callbackSignals, 0);
  PIN_InterceptSignal(SIGPIPE, callbackSignals, 0);
  PIN_InterceptSignal(SIGALRM, callbackSignals, 0);
  PIN_InterceptSignal(SIGTERM, callbackSignals, 0);
  PIN_InterceptSignal(SIGBUS,  callbackSignals, 0);

  /* Never returns */
  PIN_StartProgram();
    
  return 0;
}