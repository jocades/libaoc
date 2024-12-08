#include <stdc++.h>

int main() {
  vector<int> elfs;
  string line;
  int count = 0;
  while (getline(cin, line)) {
    if (line.empty()) {
      elfs.push_back(count);
      count = 0;
    } else {
      count += stoi(line);
    }
  }
  sort(elfs.begin(), elfs.end(), greater());
  int out = 0;
  iter(i, 0, 3) out += elfs[i];
  cout << out << endl;
}
